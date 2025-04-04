use std::{collections::BTreeSet, mem, panic};

use device::{ScreenDevice, ScreenDeviceExt};
use thedes_async_util::{
    non_blocking,
    timer::{TickParticipant, Timer},
};
use thedes_geometry::rect;
use thiserror::Error;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::{
    color::{BasicColor, Color, ColorPair},
    geometry::{Coord, CoordPair},
    grapheme,
    input::TermSizeWatch,
    mutation::{BoxedMutation, Mutation},
    runtime,
    status::Status,
    tile::{Tile, TileMutationError},
};

pub mod device;

#[derive(Debug, Error)]
#[error("Point is outside of screen canvas rectangle")]
pub struct InvalidCanvasPoint {
    #[from]
    source: rect::HorzAreaError<usize>,
}

#[derive(Debug, Error)]
#[error("Index is outside of screen canvas buffer bounds")]
pub struct InvalidCanvasIndex {
    #[from]
    source: rect::InvalidArea<usize>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to control screen device")]
    Device(
        #[from]
        #[source]
        device::Error,
    ),
    #[error("Invalid terminal point while drawing canvas, point {0}")]
    InvalidTermPoint(CoordPair),
    #[error("Invalid canvas point while drawing canvas")]
    InvalidCanvasPoint(
        #[from]
        #[source]
        InvalidCanvasPoint,
    ),
    #[error("Invalid canvas index while drawing canvas")]
    InvalidCanvasIndex(
        #[from]
        #[source]
        InvalidCanvasIndex,
    ),
    #[error("Found unknown grapheme ID")]
    UnknownGrapheme(
        #[from]
        #[source]
        grapheme::UnknownId,
    ),
    #[error("Failed to register grapheme")]
    NotGrapheme(
        #[from]
        #[source]
        grapheme::NotGrapheme,
    ),
    #[error("Failed to mutate tile contents in canvas point {0}")]
    TileMutation(CoordPair, #[source] TileMutationError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct FlushError {
    inner: non_blocking::spsc::unbounded::SendError<Vec<Command>>,
}

impl FlushError {
    fn new(
        inner: non_blocking::spsc::unbounded::SendError<Vec<Command>>,
    ) -> Self {
        Self { inner }
    }

    pub fn into_bounced_commands(self) -> Vec<Command> {
        self.inner.into_message()
    }
}

#[derive(Debug)]
pub enum Command {
    Mutation(CoordPair, Box<dyn BoxedMutation<Tile>>),
    ClearScreen(Color),
}

impl Command {
    pub fn new_mutation<M>(canvas_point: CoordPair, mutation: M) -> Self
    where
        M: Mutation<Tile>,
    {
        Self::Mutation(canvas_point, Box::new(mutation))
    }
}

#[derive(Debug)]
pub(crate) struct OpenResources {
    pub device: Box<dyn ScreenDevice>,
    pub timer: Timer,
    pub cancel_token: CancellationToken,
    pub grapheme_registry: grapheme::Registry,
    pub term_size_watch: TermSizeWatch,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Config {
    canvas_size: CoordPair,
    default_colors: ColorPair,
}

impl Config {
    pub fn new() -> Self {
        Self {
            canvas_size: CoordPair { y: 22, x: 78 },
            default_colors: ColorPair {
                background: BasicColor::Black.into(),
                foreground: BasicColor::White.into(),
            },
        }
    }

    pub fn with_canvas_size(self, size: CoordPair) -> Self {
        Self { canvas_size: size, ..self }
    }

    pub fn with_default_color(self, color: ColorPair) -> Self {
        Self { default_colors: color, ..self }
    }

    pub fn canvas_size(&self) -> CoordPair {
        self.canvas_size
    }

    pub(crate) fn open(
        self,
        mut resources: OpenResources,
        join_set: &mut runtime::JoinSet,
    ) -> ScreenHandles {
        let canvas_size = self.canvas_size;
        let (command_sender, command_receiver) =
            non_blocking::spsc::unbounded::channel();
        let status = resources.status.clone();

        join_set.spawn(async move {
            let initial_term_size = task::block_in_place(|| {
                resources.device.blocking_get_size().map_err(Error::from)
            })?;
            let renderer = Renderer::new(
                self,
                resources,
                initial_term_size,
                command_receiver,
            );
            renderer.run().await
        });

        let canvas_handle =
            CanvasHandle::new(canvas_size, status, command_sender);
        ScreenHandles { canvas: canvas_handle }
    }
}

#[derive(Debug)]
pub(crate) struct ScreenHandles {
    pub canvas: CanvasHandle,
}

#[derive(Debug)]
struct Renderer {
    device: Box<dyn ScreenDevice>,
    device_queue: Vec<device::Command>,
    ticker: TickParticipant,
    cancel_token: CancellationToken,
    term_size: CoordPair,
    canvas_size: CoordPair,
    default_colors: ColorPair,
    current_colors: ColorPair,
    current_position: CoordPair,
    working_buf: Box<[Tile]>,
    displayed_buf: Box<[Tile]>,
    dirty: BTreeSet<CoordPair>,
    grapheme_registry: grapheme::Registry,
    command_receiver: non_blocking::spsc::unbounded::Receiver<Vec<Command>>,
    term_size_watch: TermSizeWatch,
    status: Status,
}

impl Renderer {
    pub fn new(
        config: Config,
        resources: OpenResources,
        term_size: CoordPair,
        command_receiver: non_blocking::spsc::unbounded::Receiver<Vec<Command>>,
    ) -> Self {
        let tile_buf_size = usize::from(config.canvas_size.x)
            * usize::from(config.canvas_size.y);
        let tile_buf = Box::<[Tile]>::from(vec![
            Tile {
                grapheme: grapheme::Id::from(' '),
                colors: config.default_colors,
            };
            tile_buf_size
        ]);

        Self {
            device: resources.device,
            device_queue: Vec::new(),
            ticker: resources.timer.new_participant(),
            cancel_token: resources.cancel_token,
            term_size,
            canvas_size: config.canvas_size,
            default_colors: config.default_colors,
            current_colors: config.default_colors,
            current_position: CoordPair { x: 0, y: 0 },
            working_buf: tile_buf.clone(),
            displayed_buf: tile_buf,
            dirty: BTreeSet::new(),
            grapheme_registry: resources.grapheme_registry,
            command_receiver,
            term_size_watch: resources.term_size_watch,
            status: resources.status,
        }
    }

    pub async fn run(mut self) -> Result<(), runtime::Error> {
        let run_result = self.do_run().await;
        self.shutdown().await.expect("Screen shutdown failed");
        run_result
    }

    async fn do_run(&mut self) -> Result<(), runtime::Error> {
        self.init().await?;

        let mut commands = Vec::<Command>::new();

        loop {
            if !self.check_term_size_change().await? {
                break;
            }
            self.render().await?;

            tokio::select! {
                _ = self.ticker.tick() => (),
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("Screen token cancellation detected");
                    break
                },
            }

            if !self.check_term_size_change().await? {
                break;
            }
            if !self.execute_commands_sent(&mut commands)? {
                break;
            }

            tokio::select! {
                _ = self.ticker.tick() => (),
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("Screen token cancellation detected");
                    break
                },
            }
        }

        Ok(())
    }

    fn queue(&mut self, commands: impl IntoIterator<Item = device::Command>) {
        self.device_queue.extend(commands);
    }

    async fn flush(&mut self) -> Result<(), Error> {
        let _ = self.device.send(self.device_queue.drain(..));
        self.device.flush().await?;
        Ok(())
    }

    async fn init(&mut self) -> Result<(), Error> {
        self.enter()?;
        let term_size = self.term_size;
        self.status.set_blocked_from_sizes(self.canvas_size, self.term_size);
        self.term_size_changed(term_size).await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Error> {
        self.leave();
        self.flush().await?;
        Ok(())
    }

    async fn render(&mut self) -> Result<(), Error> {
        if !self.needs_resize() {
            self.draw_working_canvas()?;
            self.flush().await?;
        }
        Ok(())
    }

    pub fn needs_resize(&self) -> bool {
        self.status.is_blocked()
    }

    fn draw_working_canvas(&mut self) -> Result<(), Error> {
        for canvas_point in mem::take(&mut self.dirty) {
            let tile = self.get(canvas_point)?;
            let term_point = self.canvas_to_term(canvas_point);
            self.draw_tile(term_point, tile)?;
        }
        self.displayed_buf.clone_from(&self.working_buf);
        Ok(())
    }

    fn enter(&mut self) -> Result<(), Error> {
        self.queue([device::Command::Enter, device::Command::HideCursor]);
        Ok(())
    }

    fn leave(&mut self) {
        self.queue([
            device::Command::ShowCursor,
            device::Command::ResetBackground,
            device::Command::ResetForeground,
            device::Command::Clear,
            device::Command::Leave,
        ]);
    }

    async fn check_term_size_change(&mut self) -> Result<bool, Error> {
        let Ok(term_size_message) = self.term_size_watch.recv() else {
            tracing::info!("Terminal size watch sender disconnected");
            return Ok(false);
        };
        if let Some(new_term_size) = term_size_message {
            self.term_size_changed(new_term_size).await?;
        }
        Ok(true)
    }

    async fn term_size_changed(
        &mut self,
        new_term_size: CoordPair,
    ) -> Result<(), Error> {
        self.status.set_blocked_from_sizes(self.canvas_size, new_term_size);

        let position = self.current_position;
        self.move_to(position)?;
        let colors = self.current_colors;
        self.change_colors(colors)?;
        self.dirty.clear();
        let space = grapheme::Id::from(' ');

        for (i, (working, displayed)) in self
            .working_buf
            .iter_mut()
            .zip(&mut self.displayed_buf[..])
            .enumerate()
        {
            *displayed = Tile { grapheme: space, colors: self.default_colors };
            if *displayed != *working {
                let point = self
                    .canvas_size
                    .map(usize::from)
                    .as_rect_size(thedes_geometry::CoordPair::from_axes(|_| 0))
                    .checked_bot_right_of_horz_area(&i)
                    .map_err(InvalidCanvasIndex::from)?
                    .map(|a| a as Coord);
                self.dirty.insert(point);
            }
        }

        self.term_size = new_term_size;
        if self.needs_resize() {
            self.draw_resize_msg()?;
        } else {
            self.draw_reset()?;
        }

        self.flush().await?;

        Ok(())
    }

    fn move_to(&mut self, term_point: CoordPair) -> Result<(), Error> {
        if term_point.zip2(self.term_size).any(|(point, size)| point >= size) {
            Err(Error::InvalidTermPoint(term_point))?
        }
        self.queue([device::Command::MoveCursor(term_point)]);
        self.current_position = term_point;
        Ok(())
    }

    fn change_colors(&mut self, colors: ColorPair) -> Result<(), Error> {
        self.change_foreground(colors.foreground)?;
        self.change_background(colors.background)?;
        Ok(())
    }

    fn change_foreground(&mut self, color: Color) -> Result<(), Error> {
        self.queue([device::Command::SetForeground(color)]);
        self.current_colors.foreground = color;
        Ok(())
    }

    fn change_background(&mut self, color: Color) -> Result<(), Error> {
        self.queue([device::Command::SetBackground(color)]);
        self.current_colors.background = color;
        Ok(())
    }

    fn clear_term(&mut self, background: Color) -> Result<(), Error> {
        if background != self.current_colors.background {
            self.change_background(background)?;
        }
        self.queue([device::Command::Clear]);
        Ok(())
    }

    fn draw_reset_hor_line(
        &mut self,
        y: Coord,
        x_start: Coord,
        x_end: Coord,
    ) -> Result<(), Error> {
        let tile = Tile {
            colors: self.default_colors,
            grapheme: self.grapheme_registry.get_or_register("━")?,
        };
        for x in x_start .. x_end {
            self.draw_tile(CoordPair { x, y }, tile)?;
        }
        Ok(())
    }

    fn draw_reset(&mut self) -> Result<(), Error> {
        self.move_to(CoordPair { y: 0, x: 0 })?;
        self.clear_term(self.default_colors.background)?;

        let margin_top_left = self.top_left_margin();
        let margin_bottom_right = self.bottom_right_margin();

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┏")?,
            colors: self.default_colors,
        };
        self.draw_tile(margin_top_left - 1, tile)?;
        self.draw_reset_hor_line(
            margin_top_left.y - 1,
            margin_top_left.x,
            margin_bottom_right.x,
        )?;
        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┓")?,
            colors: self.default_colors,
        };
        self.draw_tile(
            CoordPair { x: margin_bottom_right.x, y: margin_top_left.y - 1 },
            tile,
        )?;

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┃")?,
            colors: self.default_colors,
        };
        for y in margin_top_left.y .. margin_bottom_right.y {
            self.draw_tile(CoordPair { x: margin_top_left.x - 1, y }, tile)?;
            self.draw_tile(CoordPair { x: margin_bottom_right.x, y }, tile)?;
        }

        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┗")?,
            colors: self.default_colors,
        };
        self.draw_tile(
            CoordPair { x: margin_top_left.x - 1, y: margin_bottom_right.y },
            tile,
        )?;
        self.draw_reset_hor_line(
            margin_bottom_right.y,
            margin_top_left.x,
            margin_bottom_right.x,
        )?;
        let tile = Tile {
            grapheme: self.grapheme_registry.get_or_register("┛")?,
            colors: self.default_colors,
        };
        self.draw_tile(margin_bottom_right, tile)?;

        Ok(())
    }

    fn draw_resize_msg(&mut self) -> Result<(), Error> {
        let graphemes: Vec<_> = self
            .grapheme_registry
            .get_or_register_many(&format!(
                "RESIZE {}x{}",
                self.canvas_size.x + 2,
                self.canvas_size.y + 2
            ))
            .collect();
        self.move_to(CoordPair { y: 0, x: 0 })?;
        self.clear_term(self.default_colors.background)?;
        for (grapheme, i) in graphemes.into_iter().zip(0 ..) {
            self.draw_tile(
                CoordPair { x: i, y: 0 },
                Tile { colors: self.default_colors, grapheme },
            )?;
        }
        Ok(())
    }

    fn draw_tile(
        &mut self,
        term_point: CoordPair,
        tile: Tile,
    ) -> Result<(), Error> {
        if self.current_position != term_point {
            self.move_to(term_point)?;
        }
        if self.current_colors.foreground != tile.colors.foreground {
            self.change_foreground(tile.colors.foreground)?;
        }
        if self.current_colors.background != tile.colors.background {
            self.change_background(tile.colors.background)?;
        }
        self.draw_grapheme(tile.grapheme)?;
        Ok(())
    }

    fn draw_grapheme(&mut self, id: grapheme::Id) -> Result<(), Error> {
        self.grapheme_registry.lookup(id, |result| {
            result.map(|chars| {
                self.device_queue.extend(chars.map(device::Command::Write))
            })
        })?;
        self.current_position.x += 1;
        if self.current_position.x >= self.term_size.x {
            self.current_position.x = 0;
            self.current_position.y += 1;
            if self.current_position.y >= self.term_size.y {
                self.move_to(CoordPair { y: 0, x: 0 })?;
            }
        }
        Ok(())
    }

    fn top_left_margin(&self) -> CoordPair {
        (self.term_size - self.canvas_size + 1) / 2
    }

    fn bottom_right_margin(&self) -> CoordPair {
        self.top_left_margin() + self.canvas_size
    }

    fn get(&self, canvas_point: CoordPair) -> Result<Tile, InvalidCanvasPoint> {
        let index = self.point_to_index(canvas_point)?;
        Ok(self.working_buf[index])
    }

    fn set(
        &mut self,
        canvas_point: CoordPair,
        tile: Tile,
    ) -> Result<Tile, InvalidCanvasPoint> {
        let index = self.point_to_index(canvas_point)?;
        if tile == self.displayed_buf[index] {
            self.dirty.remove(&canvas_point);
        } else {
            self.dirty.insert(canvas_point);
        }
        let old = mem::replace(&mut self.working_buf[index], tile);
        Ok(old)
    }

    fn execute_commands_sent(
        &mut self,
        buf: &mut Vec<Command>,
    ) -> Result<bool, Error> {
        let Ok(command_iterator) = self.command_receiver.recv_many() else {
            tracing::info!("Screen command sender disconnected");
            return Ok(false);
        };
        buf.extend(command_iterator.flatten());
        for command in buf.drain(..) {
            self.execute_command(command)?;
        }
        Ok(true)
    }

    fn execute_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::Mutation(canvas_point, mutation) => {
                self.execute_mutation(canvas_point, mutation)
            },
            Command::ClearScreen(color) => self.execute_clear_screen(color),
        }
    }

    fn execute_mutation(
        &mut self,
        canvas_point: CoordPair,
        mutation: Box<dyn BoxedMutation<Tile>>,
    ) -> Result<(), Error> {
        let curr_tile = self.get(canvas_point)?;
        let new_tile = mutation
            .mutate_boxed(curr_tile)
            .map_err(|source| Error::TileMutation(canvas_point, source))?;
        self.set(canvas_point, new_tile)?;
        Ok(())
    }

    fn execute_clear_screen(&mut self, color: Color) -> Result<(), Error> {
        for y in 0 .. self.canvas_size.y {
            for x in 0 .. self.canvas_size.x {
                self.set(
                    CoordPair { y, x },
                    Tile {
                        colors: ColorPair {
                            background: color,
                            foreground: color,
                        },
                        grapheme: ' '.into(),
                    },
                )?;
            }
        }
        Ok(())
    }

    fn point_to_index(
        &self,
        canvas_point: CoordPair,
    ) -> Result<usize, InvalidCanvasPoint> {
        let index = self
            .canvas_size
            .map(usize::from)
            .as_rect_size(thedes_geometry::CoordPair::from_axes(|_| 0))
            .checked_horz_area_down_to(canvas_point.map(usize::from))?;
        Ok(index)
    }

    fn canvas_to_term(&self, canvas_point: CoordPair) -> CoordPair {
        canvas_point + self.top_left_margin()
    }
}

#[derive(Debug)]
pub struct CanvasHandle {
    size: CoordPair,
    status: Status,
    command_sender: non_blocking::spsc::unbounded::Sender<Vec<Command>>,
    command_queue: Vec<Command>,
}

impl CanvasHandle {
    fn new(
        canvas_size: CoordPair,
        status: Status,
        command_sender: non_blocking::spsc::unbounded::Sender<Vec<Command>>,
    ) -> Self {
        Self {
            size: canvas_size,
            status,
            command_sender,
            command_queue: Vec::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.command_sender.is_connected()
    }

    pub fn size(&self) -> CoordPair {
        self.size
    }

    pub fn queue<I>(&mut self, commands: I)
    where
        I: IntoIterator<Item = Command>,
    {
        self.command_queue.extend(commands);
    }

    pub fn flush(&mut self) -> Result<(), FlushError> {
        let commands = mem::take(&mut self.command_queue);
        self.command_sender.send(commands).map_err(FlushError::new)
    }

    pub fn is_blocked(&self) -> bool {
        self.status.is_blocked()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use thedes_async_util::{non_blocking, timer::Timer};
    use tokio::time::timeout;
    use tokio_util::sync::CancellationToken;

    use crate::{
        color::{BasicColor, ColorPair},
        geometry::CoordPair,
        grapheme,
        input::TermSizeWatch,
        mutation::{MutationExt, Set},
        runtime::JoinSet,
        screen::{
            Command,
            Config,
            OpenResources,
            device::{self, mock::ScreenDeviceMock},
        },
        status::Status,
        tile::{MutateColors, MutateGrapheme},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn init_sends_command() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();
        assert_ne!(command_log, &[] as &[Vec<device::Command>]);

        assert!(
            command_log.iter().flatten().next().is_some(),
            "commands: {command_log:#?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn init_flushes() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();
        assert!(command_log.len() > 1);
        assert!(
            command_log.iter().flatten().next().is_some(),
            "commands: {command_log:#?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn mutation_sends_command() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let mut handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        handles.canvas.queue([Command::new_mutation(
            CoordPair { y: 12, x: 30 },
            MutateColors(Set(ColorPair {
                foreground: BasicColor::DarkCyan.into(),
                background: BasicColor::LightGreen.into(),
            }))
            .then(MutateGrapheme(Set('B'.into()))),
        )]);
        handles.canvas.flush().unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command == device::Command::Write('B'))
                .count(),
            "commands: {command_log:#?}",
        );

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command
                    == device::Command::SetForeground(
                        BasicColor::DarkCyan.into()
                    ))
                .count(),
            "commands: {command_log:#?}",
        );

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command
                    == device::Command::SetBackground(
                        BasicColor::LightGreen.into()
                    ))
                .count(),
            "commands: {command_log:#?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn clear_screen_sends_command() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let mut handles = Config::new()
            .with_canvas_size(CoordPair { y: 20, x: 60 })
            .open(resources, &mut join_set);
        handles
            .canvas
            .queue([Command::ClearScreen(BasicColor::DarkRed.into())]);
        handles.canvas.flush().unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();

        assert_eq!(
            20 * 60,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command == device::Command::Write(' '))
                .count(),
            "commands: {command_log:#?}",
        );

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command
                    == device::Command::SetBackground(
                        BasicColor::DarkRed.into()
                    ))
                .count(),
            "commands: {command_log:#?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resize_too_small() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (mut term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let handles = Config::new()
            .with_canvas_size(CoordPair { y: 24, x: 80 })
            .open(resources, &mut join_set);
        term_size_sender.send(CoordPair { y: 26, x: 81 }).unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        assert!(handles.canvas.is_blocked());
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();

        let resize_msg = "RESIZE 82x26";
        for ch in resize_msg.chars() {
            assert_eq!(
                resize_msg.chars().filter(|resize_ch| *resize_ch == ch).count(),
                command_log
                    .iter()
                    .flatten()
                    .filter(|command| **command == device::Command::Write(ch))
                    .count(),
                "expected {ch} to occur once, commands: {command_log:#?}",
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resize_min_size() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (mut term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let mut handles = Config::new()
            .with_canvas_size(CoordPair { y: 24, x: 80 })
            .open(resources, &mut join_set);
        term_size_sender.send(CoordPair { y: 26, x: 82 }).unwrap();
        handles.canvas.queue([Command::new_mutation(
            CoordPair { y: 12, x: 30 },
            MutateColors(Set(ColorPair {
                foreground: BasicColor::DarkCyan.into(),
                background: BasicColor::LightGreen.into(),
            }))
            .then(MutateGrapheme(Set('B'.into()))),
        )]);
        handles.canvas.flush().unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();

        let resize_msg = "RESIZE 82x26";
        let mut occurences = 0;

        for ch in resize_msg.chars() {
            if command_log
                .iter()
                .flatten()
                .filter(|command| **command == device::Command::Write(ch))
                .count()
                >= resize_msg
                    .chars()
                    .filter(|resize_ch| *resize_ch == ch)
                    .count()
            {
                occurences += 1;
            }
        }

        assert_ne!(occurences, resize_msg.len(), "commands: {command_log:#?}",);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn mutation_sends_command_after_resize_correction() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let mut tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (mut term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token,
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let mut handles = Config::new()
            .with_canvas_size(CoordPair { y: 24, x: 80 })
            .open(resources, &mut join_set);
        term_size_sender.send(CoordPair { y: 26, x: 81 }).unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        term_size_sender.send(CoordPair { y: 26, x: 82 }).unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        handles.canvas.queue([Command::new_mutation(
            CoordPair { y: 12, x: 30 },
            MutateColors(Set(ColorPair {
                foreground: BasicColor::DarkCyan.into(),
                background: BasicColor::LightGreen.into(),
            }))
            .then(MutateGrapheme(Set('B'.into()))),
        )]);
        handles.canvas.flush().unwrap();
        tick_participant.tick().await;
        tick_participant.tick().await;
        drop(tick_participant);
        drop(handles);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }

        let command_log = device_mock.take_command_log().unwrap();

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command == device::Command::Write('B'))
                .count(),
            "commands: {command_log:#?}",
        );

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command
                    == device::Command::SetForeground(
                        BasicColor::DarkCyan.into()
                    ))
                .count(),
            "commands: {command_log:#?}",
        );

        assert_eq!(
            1,
            command_log
                .iter()
                .flatten()
                .filter(|command| **command
                    == device::Command::SetBackground(
                        BasicColor::LightGreen.into()
                    ))
                .count(),
            "commands: {command_log:#?}",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_token_stops() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token: cancel_token.clone(),
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let _handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        drop(tick_participant);
        cancel_token.cancel();

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drop_term_size_stops() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token: cancel_token.clone(),
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let _handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        drop(tick_participant);
        drop(term_size_sender);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drop_canvas_handle_stops() {
        let device_mock = ScreenDeviceMock::new(CoordPair { y: 40, x: 90 });
        device_mock.enable_command_log();
        let device = device_mock.open();

        let timer = Timer::new(Duration::from_millis(4));
        let tick_participant = timer.new_participant();
        let cancel_token = CancellationToken::new();
        let grapheme_registry = grapheme::Registry::new();
        let (_term_size_sender, term_size_receiver) =
            non_blocking::spsc::watch::channel();
        let term_size_watch = TermSizeWatch::new(term_size_receiver);

        let mut join_set = JoinSet::new();

        let resources = OpenResources {
            device,
            timer,
            cancel_token: cancel_token.clone(),
            grapheme_registry,
            term_size_watch,
            status: Status::new(),
        };

        let handles = Config::new()
            .with_canvas_size(CoordPair { y: 22, x: 78 })
            .open(resources, &mut join_set);
        drop(tick_participant);
        drop(handles.canvas);

        let results = timeout(Duration::from_millis(200), join_set.join_all())
            .await
            .unwrap();
        for result in results {
            result.unwrap();
        }
    }
}

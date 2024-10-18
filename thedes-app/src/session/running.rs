use num::rational::Ratio;
use thedes_domain::{
    block::{Block, PlaceableBlock},
    game::{Game, MovePlayerError},
    item::{self, Inventory, SlotEntry, StackableEntry8, StackableItem8},
    map::AccessError,
};
use thedes_gen::game;
use thedes_geometry::axis::Direction;
use thedes_graphics::{
    game_screen::{self, GameScreen},
    SessionData,
};
use thedes_tui::{
    event::{Event, Key, KeyEvent},
    Tick,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to generate game")]
    Gen(
        #[from]
        #[source]
        game::GenError,
    ),
}

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    RenderError(#[from] thedes_tui::CanvasError),
    #[error("Error happened while rendering game on-camera")]
    GameScreen(
        #[from]
        #[source]
        game_screen::Error,
    ),
    #[error("Failed to control player")]
    MovePlayer(
        #[from]
        #[source]
        MovePlayerError,
    ),
    #[error("Failed to access inventory slot")]
    InventoryAccess(
        #[from]
        #[source]
        item::AccessError,
    ),
    #[error("Failed updating inventory")]
    InvalidItemCount(
        #[from]
        #[source]
        item::InvalidCount,
    ),
    #[error("Failed to access map positions")]
    AccessError(
        #[from]
        #[source]
        AccessError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Pause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum EventAction {
    Propagate(Action),
    Control(ControlAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ControlAction {
    MovePlayerHead(Direction),
    MovePlayerPointer(Direction),
    Activate,
    InventoryDrop,
    InventoryUp,
    InventoryDown,
}

fn arrow_key_to_direction(key: Key) -> Option<Direction> {
    Some(match key {
        Key::Up => Direction::Up,
        Key::Left => Direction::Left,
        Key::Down => Direction::Down,
        Key::Right => Direction::Right,
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub struct Component {
    first_render: bool,
    control_events_per_tick: Ratio<u64>,
    controls_left: Ratio<u64>,
    game_screen: GameScreen,
    game: Game,
    selected_inventory_slot: usize,
}

impl Component {
    pub fn new(game: Game) -> Result<Self, InitError> {
        let control_events_per_tick = Ratio::new(1, 8);

        Ok(Self {
            first_render: true,
            control_events_per_tick,
            controls_left: control_events_per_tick,
            game_screen: game_screen::Config::default().finish(),
            game,
            selected_inventory_slot: 0,
        })
    }

    pub fn reset(&mut self) {
        self.first_render = true;
        self.selected_inventory_slot = 0;
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        if !self.first_render {
            if let Some(action) = self.handle_input(tick)? {
                return Ok(Some(action));
            }
        }
        let more_controls_left =
            self.controls_left + self.control_events_per_tick;
        if more_controls_left < self.control_events_per_tick.ceil() * 2 {
            self.controls_left = more_controls_left;
        }
        let session_data = SessionData {
            selected_inventory_slot: self.selected_inventory_slot,
        };
        self.game_screen.on_tick(tick, &self.game, &session_data)?;
        self.first_render = false;
        self.game.on_post_tick();
        Ok(None)
    }

    fn handle_input(
        &mut self,
        tick: &mut Tick,
    ) -> Result<Option<Action>, TickError> {
        while let Some(event) = tick.next_event() {
            if let Some(event_action) = self.handle_input_event(event)? {
                match event_action {
                    EventAction::Propagate(action) => return Ok(Some(action)),
                    EventAction::Control(action) => {
                        if self.controls_left >= Ratio::ONE {
                            self.controls_left -= Ratio::ONE;
                            self.handle_control(action)?;
                        }
                    },
                }
            }
        }
        Ok(None)
    }

    fn handle_input_event(
        &mut self,
        event: Event,
    ) -> Result<Option<EventAction>, TickError> {
        match event {
            Event::Key(
                KeyEvent {
                    main_key: Key::Char('p') | Key::Char('P'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
                | KeyEvent { main_key: Key::Esc, .. },
            ) => Ok(Some(EventAction::Propagate(Action::Pause))),

            Event::Key(KeyEvent {
                main_key,
                ctrl,
                alt: false,
                shift: false,
            }) => {
                if let Some(direction) = arrow_key_to_direction(main_key) {
                    let action = if ctrl {
                        ControlAction::MovePlayerHead(direction)
                    } else {
                        ControlAction::MovePlayerPointer(direction)
                    };
                    Ok(Some(EventAction::Control(action)))
                } else if !ctrl && main_key == Key::Char(' ') {
                    Ok(Some(EventAction::Control(ControlAction::Activate)))
                } else if !ctrl && main_key == Key::Char('s') {
                    Ok(Some(EventAction::Control(ControlAction::InventoryUp)))
                } else if !ctrl && main_key == Key::Char('a') {
                    Ok(Some(EventAction::Control(ControlAction::InventoryDown)))
                } else if !ctrl && main_key == Key::Char('x') {
                    Ok(Some(EventAction::Control(ControlAction::InventoryDrop)))
                } else {
                    Ok(None)
                }
            },

            _ => Ok(None),
        }
    }

    fn handle_control(
        &mut self,
        action: ControlAction,
    ) -> Result<(), TickError> {
        match action {
            ControlAction::MovePlayerHead(direction) => {
                self.game.move_player_head(direction)?;
            },
            ControlAction::MovePlayerPointer(direction) => {
                self.game.move_player_pointer(direction)?;
            },

            ControlAction::InventoryUp => {
                self.selected_inventory_slot =
                    self.selected_inventory_slot.saturating_sub(1);
            },
            ControlAction::InventoryDown => {
                let next_slot = self.selected_inventory_slot + 1;
                self.selected_inventory_slot =
                    next_slot.min(Inventory::SLOT_COUNT - 1);
            },

            ControlAction::InventoryDrop => {
                let pointer_pos = self.game.player().position().pointer();
                let facing_direction = self.game.player().position().facing();
                if let Ok(target_point) = self
                    .game
                    .map()
                    .rect()
                    .checked_move_point_unit(pointer_pos, facing_direction)
                {
                    let stored = self
                        .game
                        .player()
                        .inventory()
                        .get(self.selected_inventory_slot)?;
                    match (self.game.map().get_block(target_point)?, stored) {
                        (
                            Block::Placeable(PlaceableBlock::Air),
                            SlotEntry::Stackable8(entry),
                        ) if entry.item() == StackableItem8::Stick => {
                            if let Ok(new_entry) = StackableEntry8::new(
                                entry.item(),
                                entry.count() - 1,
                            ) {
                                self.game.player_picked(
                                    self.selected_inventory_slot,
                                    SlotEntry::Stackable8(new_entry),
                                )?;
                            } else {
                                self.game.player_picked(
                                    self.selected_inventory_slot,
                                    SlotEntry::Vaccant,
                                )?;
                            }
                            self.game.place_block(
                                target_point,
                                PlaceableBlock::Stick,
                            )?;
                        },

                        _ => (),
                    }
                }
            },

            ControlAction::Activate => {
                let pointer_pos = self.game.player().position().pointer();
                let facing_direction = self.game.player().position().facing();
                if let Ok(target_point) = self
                    .game
                    .map()
                    .rect()
                    .checked_move_point_unit(pointer_pos, facing_direction)
                {
                    let stored = self
                        .game
                        .player()
                        .inventory()
                        .get(self.selected_inventory_slot)?;
                    match (self.game.map().get_block(target_point)?, stored) {
                        (
                            Block::Placeable(PlaceableBlock::Stick),
                            SlotEntry::Stackable8(entry),
                        ) if entry.item() == StackableItem8::Stick => {
                            if let Ok(new_entry) = StackableEntry8::new(
                                entry.item(),
                                entry.count() + 1,
                            ) {
                                self.game.place_block(
                                    target_point,
                                    PlaceableBlock::Air,
                                )?;
                                self.game.player_picked(
                                    self.selected_inventory_slot,
                                    SlotEntry::Stackable8(new_entry),
                                )?;
                            }
                        },

                        (
                            Block::Placeable(PlaceableBlock::Stick),
                            SlotEntry::Vaccant,
                        ) => {
                            if let Ok(new_entry) =
                                StackableEntry8::new(StackableItem8::Stick, 1)
                            {
                                self.game.place_block(
                                    target_point,
                                    PlaceableBlock::Air,
                                )?;
                                self.game.player_picked(
                                    self.selected_inventory_slot,
                                    SlotEntry::Stackable8(new_entry),
                                )?;
                            }
                        },

                        _ => (),
                    }
                }
            },
        }
        Ok(())
    }
}

use andiskaz::{
    coord::Coord as TermCoord,
    string::TermString,
    terminal::{self, Terminal},
    ui::{
        input::InputDialog,
        menu::{Menu, MenuOption},
    },
};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{
    domain::PlayerName,
    error::Result,
    message::{self, LoginRequest, LoginResponse},
};

const MAX_ADDRESS_SIZE: TermCoord = 45 + 6;

const MAX_NAME_SIZE: TermCoord = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuOption {
    Connect,
    Exit,
}

impl MenuOption for MainMenuOption {
    fn name(&self) -> TermString {
        TermString::new_lossy(match self {
            Self::Connect => "connect",
            Self::Exit => "exit",
        })
    }
}

pub async fn start() -> Result<()> {
    terminal::Builder::new().run(run_launcher).await??;
    Ok(())
}

async fn run_launcher(mut terminal: Terminal) -> Result<()> {
    let mut name_input = InputDialog::new(
        TermString::new_lossy("Connect to..."),
        TermString::new_lossy(""),
        MAX_NAME_SIZE,
        |ch| {
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-$^#@:.%"
                .contains(ch)
        },
    );
    let mut main_menu = Menu::new(
        TermString::new_lossy("T H E D E S"),
        vec![MainMenuOption::Connect, MainMenuOption::Exit],
    );
    let mut connect_input = InputDialog::new(
        TermString::new_lossy("Connect to..."),
        TermString::new_lossy(""),
        MAX_ADDRESS_SIZE,
        |_| true,
    );
    while let Some(term_name) =
        name_input.select_with_cancel(&mut terminal).await?
    {
        let player_name = term_name.to_string();
        run_main_menu(
            &mut terminal,
            &mut main_menu,
            &mut connect_input,
            player_name,
        )
        .await?;
    }
    Ok(())
}

async fn run_main_menu<F>(
    terminal: &mut Terminal,
    menu: &mut Menu<MainMenuOption>,
    connect_input: &mut InputDialog<F>,
    player_name: PlayerName,
) -> Result<()>
where
    F: FnMut(char) -> bool,
{
    loop {
        let index = menu.select(terminal).await?;
        match menu.options[index] {
            MainMenuOption::Connect => {
                run_connect_ui(terminal, connect_input, player_name.clone())
                    .await?
            },
            MainMenuOption::Exit => break,
        }
    }
    Ok(())
}

async fn run_connect_ui<F>(
    terminal: &mut Terminal,
    input: &mut InputDialog<F>,
    player_name: PlayerName,
) -> Result<()>
where
    F: FnMut(char) -> bool,
{
    if let Some(address) = input.select_with_cancel(terminal).await? {
        todo!()
    }
    Ok(())
}

async fn run_server<S>(
    server_addr: S,
    player_name: PlayerName,
    terminal: &mut Terminal,
) -> Result<()>
where
    S: ToSocketAddrs,
{
    let mut connection = TcpStream::connect(server_addr).await?;

    let login_request = LoginRequest { player_name };
    message::send(&mut connection, login_request).await?;
    let login_response: LoginResponse =
        message::receive(&mut connection).await?;
    login_response.result?;

    loop {}
}

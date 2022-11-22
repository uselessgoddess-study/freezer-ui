#![deny(clippy::all, clippy::nursery, clippy::perf, clippy::pedantic)]
#![allow(clippy::items_after_statements)]
#![feature(io_error_other)]
#![feature(never_type)]
#![feature(default_free_fn)]
#![feature(iter_array_chunks)]
#![feature(box_syntax)]
#![feature(let_chains)]

mod model;
// todo: bug in rust plugin - useless mod
mod client;
mod utils;
mod views;

use crate::{
    alignment::Horizontal,
    client::Client,
    views::{list, preview, List, Preview},
};
use iced::{alignment, executor, Application, Command, Element, Length, Renderer, Settings};
use iced_aw::{Card, Modal};
use iced_native::widget::helpers::{button, container, horizontal_rule, row, text, text_input};
use std::{default::default, io, sync::Arc};
use tap::Pipe;
use tracing::{error, warn};
use utils::Result;

pub fn main() -> iced::Result {
    let (non_blocking, _guard) = tracing_appender::non_blocking(io::stdout());
    tracing_subscriber::fmt()
        // ---
        .with_writer(non_blocking)
        .init();

    App::run(Settings {
        window: iced::window::Settings {
            size: (1920, 800),
            resizable: true,
            transparent: true,
            decorations: true,
            ..default()
        },
        default_font: Some(include_bytes!("../fonts/JetBrainsMono-Regular.ttf")),
        default_text_size: 17,
        text_multithreading: true,
        antialiasing: true,
        ..default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    ResetInit,

    HostChanged(String),
    LoginChanged(String),
    OnLogin,

    OnLoginResponse(Result<()>),

    List(list::Message),
    Preview(preview::Message),
}

#[derive(Debug)]
enum State {
    Login,
    WaitLogin { client: Arc<Client> },
    Ready { list: List, preview: Preview },
}

struct App {
    state: State,

    host: String,
    login: String,
}

impl Application for App
where
    Self: 'static,
{
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                state: State::Login,
                host: Client::DEFAULT_API.to_owned(),
                login: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        match self.state {
            State::Login { .. } => "Login",
            State::WaitLogin { .. } => "Waiting for Login",
            State::Ready { .. } => "Ready",
        }
        .pipe(|str| format!("Freezers Client - {}", str))
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match &mut self.state {
            State::Login => match message {
                Message::HostChanged(new) => {
                    self.host = new;
                    Command::none()
                }
                Message::LoginChanged(new) => {
                    self.login = new;
                    Command::none()
                }
                Message::OnLogin => {
                    if self.login.is_empty() {
                        error!("Login cannot be empty");
                        Command::none()
                    } else {
                        let Self { host, login, .. } = self;
                        let login = login.clone();
                        let client = Arc::new(Client::new(host, reqwest::Client::new()));
                        self.state = State::WaitLogin {
                            client: client.clone(),
                        };
                        Command::perform(
                            async move { client.login(&login).await },
                            Message::OnLoginResponse,
                        )
                    }
                }
                _ => Command::none(),
            },
            State::WaitLogin { client, .. } => match &message {
                Message::ResetInit => {
                    self.state = State::Login;
                    Command::none()
                }
                Message::OnLoginResponse(res) => match res {
                    Ok(_) => {
                        let (list, preview, command) = Self::on_login(client);
                        self.state = State::Ready { list, preview };
                        command
                    }
                    Err(error) => {
                        self.state = State::Login;
                        error!(%error);
                        Command::none()
                    }
                },
                _ => Command::none(),
            },
            State::Ready { list, preview } => match message {
                Message::List(message) => {
                    let mut commands = Vec::new();

                    if let list::Message::Ping(list) = &message {
                        commands.push(
                            preview
                                .update(preview::Message::FetchRequest(list.clone()))
                                .map(Message::Preview),
                        );
                    }

                    if let list::Message::Error(error) = &message {
                        error!(%error);
                    }

                    commands.push(list.update(message).map(Message::List));

                    Command::batch(commands)
                }
                Message::Preview(message) => {
                    if let preview::Message::Error(error) = &message {
                        error!(%error);
                    }
                    if let preview::Message::Warn(error) = &message {
                        warn!(%error);
                    }
                    preview.update(message).map(Message::Preview)
                }
                _ => Command::none(),
            },
        }
    }

    // fn subscription(&self) -> Subscription<Self::Message> {
    //     time::every(Duration::from_millis(50)).map(|_| Message::Silent)
    // }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let Self { host, login, .. } = self;

        let view = match &self.state {
            State::Login | State::WaitLogin { .. } => Self::login(host, login),
            State::Ready { list, preview } => Self::ready(list, preview),
        };

        let content = container(view).height(Length::Fill);

        Modal::new(
            matches!(self.state, State::WaitLogin { .. }),
            content,
            || {
                Card::new(
                    text("Wait please..."),
                    button(text("Cancel").horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::ResetInit),
                )
                .max_width(400)
                //.width(Length::Shrink)
                .into()
            },
        )
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::Dark
    }
}

impl App {
    fn login<'a>(host: &str, login: &str) -> Element<'a, Message, Renderer<iced::Theme>> {
        columee![
            text_input("host", host, Message::HostChanged),
            text_input("login", login, Message::LoginChanged),
            button("login").on_press(Message::OnLogin)
        ]
        .into()
    }

    fn ready<'a>(
        list: &'a List,
        preview: &'a Preview,
    ) -> Element<'a, Message, Renderer<iced::Theme>> {
        row![
            container(list.view().map(Message::List)).width(Length::Units(400)),
            container(preview.view().map(Message::Preview))
        ]
        .into()
    }

    fn on_login(client: &Arc<Client>) -> (List, Preview, Command<Message>) {
        let (list, command1) = List::new(client.clone());
        let (preview, command2) = Preview::new(client.clone());
        (
            list,
            preview,
            Command::batch([command1.map(Message::List), command2.map(Message::Preview)]),
        )
    }
}

fn empty<'a, Message>() -> Element<'a, Message> {
    text("").into()
}

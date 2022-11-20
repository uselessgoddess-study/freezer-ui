use crate::{columee, utils::Result, Client};
use iced::{
    alignment::{Horizontal, Vertical},
    Element, Length, Renderer, Theme,
};
use iced_native::Command;
use std::sync::Arc;

use crate::utils::Error;
use iced_native::widget::helpers::{button, column, container, scrollable, text};

#[derive(Debug, PartialEq)]
enum State {
    Loading,
    Ready,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// `update` happens, but we do nothing
    Silent,

    Ping(String),
    PageAdd(Result<Vec<String>>),
    ScrollListEnd,

    Error(Error),
}

// todo: `.page()` config
const PAGE: usize = 30;

#[derive(Debug)]
pub struct List {
    state: State,
    freezers: Vec<String>,

    client: Arc<Client>,
}

impl List {
    async fn freezers_list(client: Arc<Client>, count: usize) -> Result<Vec<String>> {
        client.freezers_by(PAGE, count).await
    }

    pub fn new(client: Arc<Client>) -> (Self, Command<Message>) {
        (
            Self {
                state: State::Loading,
                freezers: vec![],
                client: client.clone(),
            },
            Command::perform(
                async move { Self::freezers_list(client, 0).await },
                Message::PageAdd,
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PageAdd(freezers) => match freezers {
                Ok(freezers) => {
                    self.freezers.extend(freezers);
                    self.state = State::Ready;
                    Command::none()
                }
                Err(error) => Command::perform(async move { error }, Message::Error),
            },
            Message::ScrollListEnd => {
                if self.state == State::Loading {
                    Command::none()
                } else {
                    self.state = State::Loading;
                    let client = Arc::clone(&self.client);
                    let count = self.freezers.len();
                    Command::perform(
                        async move { Self::freezers_list(client, count).await },
                        Message::PageAdd,
                    )
                }
            }
            _ => Command::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer<Theme>> {
        let line = |freezer: String| {
            button(text(freezer.clone()))
                .on_press(Message::Ping(freezer))
                .height(Length::Units(50))
                .width(Length::Fill)
                .into()
        };

        #[allow(clippy::float_cmp)]
        fn on_full_scroll(step: f32) -> Message {
            // fixme: `.on_scroll` give strong 1.0 literal on end of scroll
            if step == 1.0 {
                Message::ScrollListEnd
            } else {
                Message::Silent
            }
        }

        let list = scrollable(column(self.freezers.iter().cloned().map(line).collect()).spacing(5))
            .height(Length::FillPortion(30))
            .scrollbar_width(0)
            .scroller_width(5)
            .on_scroll(on_full_scroll);

        let head_inner: Element<Message> = if self.state == State::Loading {
            text("LOADING BITCH...")
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center)
                .into()
        } else {
            text("").into()
        };

        let head = container(head_inner)
            .height(Length::Fill)
            .width(Length::Fill);

        columee![head, list].into()
    }
}

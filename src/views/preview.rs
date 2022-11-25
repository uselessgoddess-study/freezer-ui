use crate::{columee, empty, model, row, utils::Result, Client};

use futures::try_join;
use iced::{
    alignment::Horizontal,
    widget,
    widget::{Container, Tooltip},
    Element, Length,
};
use iced_aw::{style::BadgeStyles, Badge, Card, Modal, NumberInput};
use iced_native::Command;
use std::sync::Arc;

use crate::{
    model::{Model, Product},
    utils::{error::anyio, Error},
};
use iced_native::{
    image,
    widget::{
        helpers::{button, column, container, horizontal_rule, image, scrollable, text},
        tooltip::Position,
    },
};
use tap::Pipe;

#[derive(Debug, Clone)]
pub struct Freezer {
    pub name: String,
    pub model: Model,
    pub owner: Option<String>,
    pub products: Vec<(String, usize)>,
}

impl From<Freezer> for model::Freezer {
    fn from(
        Freezer {
            name,
            model,
            owner,
            products,
        }: Freezer,
    ) -> Self {
        Self {
            name,
            model,
            owner,
            products: products.into_iter().collect(),
        }
    }
}

impl From<model::Freezer> for Freezer {
    fn from(
        model::Freezer {
            name,
            model,
            owner,
            products,
        }: model::Freezer,
    ) -> Self {
        Self {
            name,
            model,
            owner,
            products: products.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// `update` happens, but we do nothing
    Silent,
    ModalExit,

    FetchInfo(Result<(image::Handle, Freezer)>),
    FetchRequest(String),

    InputOwner(String),
    InputModel(String),
    InputYear(usize),
    InputProduct(String),
    ChangeProduct {
        id: usize,
        amount: usize,
    },

    StartAddProduct,
    OnAddProduct(Result<Option<Product>>),

    StartUpdate,
    OnUpdate(Result<Option<model::Freezer>>),

    StartDelete,
    OnDelete(Result<bool>),

    Error(Error),
    Warn(Error),
}

#[derive(Debug, Clone)]
enum State {
    Loading { on_cancel: Box<State> },
    Ready,
}

#[derive(Debug)]
pub struct Preview {
    state: State,

    info: Option<(image::Handle, Freezer)>,
    product: String,
    client: Arc<Client>,
}

impl Preview {
    async fn fetch_image(client: Arc<Client>, id: String) -> Result<image::Handle> {
        client
            .image_bytes(&id)
            .await
            .map(|b| b.as_ref().to_vec())
            .map(image::Handle::from_memory)
    }

    async fn fetch_freezer(client: Arc<Client>, id: String) -> Result<Freezer> {
        client.freezer(&id).await.map(Into::into)
    }

    async fn fetch_info(client: Arc<Client>, id: String) -> Result<(image::Handle, Freezer)> {
        try_join!(
            async { Self::fetch_image(client.clone(), id.clone()).await },
            async { Self::fetch_freezer(client.clone(), id.clone()).await }
        )
    }

    pub fn new(client: Arc<Client>) -> (Self, Command<Message>) {
        (
            Self {
                state: State::Ready,
                info: None,
                product: String::new(),
                client,
            },
            Command::none(),
        )
    }

    // fixme: use more understand state manager
    #[warn(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) -> Command<Message> {
        if let State::Ready = self.state && let Message::FetchRequest(id) = &message {
            self.state = State::Loading {
                on_cancel: box State::Ready,
            };
            let id = id.clone();
            let client = self.client.clone();
            return Command::perform(
                Self::fetch_info(client, id),
                Message::FetchInfo,
            );
        }

        match &mut self.state {
            State::Loading { on_cancel } => {
                self.state = State::Ready;
                match message {
                    Message::ModalExit => Command::none(),
                    Message::FetchInfo(Ok(info)) => {
                        self.info = Some(info);
                        Command::none()
                    }
                    Message::OnUpdate(Ok(Some(_))) => Command::none(),
                    Message::OnUpdate(Ok(None)) => Command::perform(
                        anyio!("unauthorized access - try login with high privileges"),
                        Message::Warn,
                    ),
                    Message::OnDelete(Ok(is_delete)) => {
                        if is_delete {
                            self.info = None;
                            Command::none()
                        } else {
                            Command::perform(
                                anyio!("unauthorized access - try login with high privileges"),
                                Message::Warn,
                            )
                        }
                    }
                    Message::OnAddProduct(Ok(Some(Product { name, default }))) => {
                        if let Some((_, freezer)) = &mut self.info {
                            if let Some(index) =
                                freezer.products.iter().position(|(key, _)| key == &name)
                            {
                                return Command::perform(
                                    anyio!("already exists `{name}` at `{index}`"),
                                    Message::Error,
                                );
                            }
                            freezer.products.push((name, default));
                        }
                        Command::none()
                    }
                    Message::FetchInfo(Err(error)) => {
                        Command::perform(async move { error }, Message::Error)
                    }
                    Message::OnUpdate(Err(error)) => {
                        Command::perform(async move { error }, Message::Error)
                    }
                    Message::OnDelete(Err(error)) => {
                        Command::perform(async move { error }, Message::Error)
                    }
                    Message::OnAddProduct(Ok(None)) => {
                        let product = self.product.clone();
                        Command::perform(anyio!("Not found product `{product}`"), Message::Error)
                    }
                    Message::OnAddProduct(Err(error)) => {
                        Command::perform(async move { error }, Message::Error)
                    }
                    _ => Command::none(),
                }
            }
            State::Ready => {
                if let Some((_, freezer)) = &mut self.info {
                    match message.clone() {
                        Message::InputOwner(owner) => {
                            freezer.owner = Some(owner);
                        }
                        Message::InputModel(model) => {
                            freezer.model.name = model;
                        }
                        Message::InputYear(year) => {
                            freezer.model.year = year;
                        }
                        Message::ChangeProduct { id, amount } => {
                            freezer.products[id].1 = amount;
                        }
                        Message::InputProduct(product) => {
                            self.product = product;
                        }
                        Message::StartUpdate => {
                            self.state = State::Loading {
                                on_cancel: box State::Ready,
                            };
                            let client = self.client.clone();
                            let freezer = freezer.clone();
                            return Command::perform(
                                async move { client.update_freezer(freezer.into()).await },
                                Message::OnUpdate,
                            );
                        }
                        Message::StartDelete => {
                            self.state = State::Loading {
                                on_cancel: box State::Ready,
                            };
                            let client = self.client.clone();
                            let id = freezer.name.clone();
                            return Command::perform(
                                async move { client.delete_freezer(&id).await },
                                Message::OnDelete,
                            );
                        }
                        _ => {}
                    }
                }
                match message {
                    Message::StartAddProduct => {
                        self.state = State::Loading {
                            on_cancel: box State::Ready,
                        };
                        let product = self.product.clone();
                        let client = self.client.clone();
                        Command::perform(
                            async move { client.product(&product).await },
                            Message::OnAddProduct,
                        )
                    }
                    _ => Command::none(),
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let view = if let Some((image, freezer)) = self.info.clone() {
            Self::ready(&self.product, image, freezer)
        } else {
            empty()
        };

        Modal::new(
            matches!(self.state, State::Loading { .. }),
            container(view).width(Length::Shrink),
            || {
                Card::new(
                    text("Wait please..."),
                    button(text("Cancel").horizontal_alignment(Horizontal::Center))
                        .width(Length::Fill)
                        .on_press(Message::ModalExit),
                )
                .max_width(300)
                //.width(Length::Shrink)
                .into()
            },
        )
        .into()
    }

    fn ready<'a>(product: &str, handle: image::Handle, freezer: Freezer) -> Element<'a, Message> {
        pub fn tooltip<'a, Message: Clone + 'a>(
            tip: impl ToString,
            content: impl Into<Element<'a, Message>>,
        ) -> Tooltip<'a, Message> {
            widget::tooltip(content, tip, Position::FollowCursor)
        }

        let Freezer {
            name,
            model: Model { name: model, year },
            owner,
            products,
        } = freezer;

        pub fn text_input<'a, Message: Clone + 'a>(
            place: &str,
            value: &str,
            on_change: impl Fn(String) -> Message + 'a,
        ) -> widget::TextInput<'a, Message> {
            widget::TextInput::new(place, value, on_change)
        }

        let number_input = |place, on_change| NumberInput::new(place, 2022, on_change).min(1999);

        pub fn info<'a, Message: Clone + 'a>(
            content: impl Into<Element<'a, Message>>,
        ) -> Container<'a, Message> {
            Container::new(content).width(Length::Fill).padding(5)
        }

        let content = columee![
            // todo: fix max width (possible before 800)
            container(image(handle)).max_height(400),
            tooltip(
                "name",
                text_input("Name cannot be empty", &name, |_| Message::Silent)
            )
            .pipe(info),
            tooltip(
                "owner",
                text_input("None", &owner.unwrap_or_default(), Message::InputOwner)
            )
            .pipe(info),
            tooltip(
                "model",
                text_input("Name cannot be empty", &model, Message::InputModel)
            )
            .pipe(info),
            tooltip("year", number_input(year, Message::InputYear)).pipe(info),
            columee![
                row![
                    text("PRODUCTS").size(40),
                    text_input("new product", product, Message::InputProduct)
                        .on_submit(Message::StartAddProduct)
                ],
                column(
                    products
                        .into_iter()
                        .enumerate()
                        .map(|(id, (product, amount))| Badge::new(row![
                            text(product).size(25),
                            NumberInput::new(amount, usize::MAX, move |amount| {
                                Message::ChangeProduct { id, amount }
                            })
                        ])
                        .style(BadgeStyles::Info)
                        .into())
                        .collect()
                )
                .width(Length::Fill) //.height(Length::Fill)
            ],
            horizontal_rule(10),
            row![
                button("UPDATE").on_press(Message::StartUpdate),
                button("DELETE").on_press(Message::StartDelete),
            ]
        ];

        columee![scrollable(content).height(Length::FillPortion(30))].into()
    }
}

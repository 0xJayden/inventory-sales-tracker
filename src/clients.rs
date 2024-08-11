use std::env;

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, Row, Scrollable, Text},
    Alignment, Element, Length,
};
use sqlx::SqlitePool;

use crate::{
    components::{
        add_button, layout, table_column, table_header, table_row_style, table_style,
        text_input_column, CustomButtonStyle, CustomMainButtonStyle,
    },
    error::Errorr,
    AppMessage,
};

#[derive(Default, Clone, Debug)]
pub struct Client {
    pub client_id: i64,
    pub name: String,
    pub email: Option<String>,
    pub address: String,
}

#[derive(Default, Clone)]
pub struct ClientState {
    pub clients: Vec<Client>,
    add_client: bool,
    pub client_to_add: Client,
    pub edit_client: bool,
    pub client_to_edit: Client,
}

#[derive(Clone, Debug)]
pub enum ClientMessage {
    NameInput(String, bool),
    AddressInput(String, bool),
    EmailInput(String, bool),
    Submit(bool),
    ShowAddClient,
    Delete,
}

pub async fn get_clients() -> Result<Vec<Client>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let clients = sqlx::query_as!(Client, "SELECT * FROM Client")
        .fetch_all(&pool)
        .await?;

    Ok(clients)
}

pub async fn get_client(i: i64) -> Result<Client, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let client = sqlx::query_as!(Client, "SELECT * FROM Client WHERE client_id = ?", i)
        .fetch_one(&pool)
        .await?;

    Ok(client)
}

pub async fn add_client(client: Client) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let name = client.name;
    let address = client.address;
    let email = client.email;

    sqlx::query!(
        "
        INSERT INTO Client (name, address, email)
        VALUES (?,?,?)
        ",
        name,
        address,
        email,
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn edit_client(client: Client) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let id = client.client_id;
    let name = client.name;
    let address = client.address;
    let email = client.email;

    sqlx::query!(
        "
        UPDATE Client
        SET name = ?, address = ?, email = ?
        WHERE client_id = ?
        ",
        name,
        address,
        email,
        id
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn delete_client(client: Client) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let id = client.client_id;

    sqlx::query!(
        "
        DELETE FROM Client
        WHERE client_id = ?
        ",
        id
    )
    .execute(&pool)
    .await?;

    Ok(())
}

impl ClientState {
    pub fn update(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::NameInput(s, is_edit) => {
                if is_edit {
                    self.client_to_edit.name = s;
                } else {
                    self.client_to_add.name = s;
                }
            }
            ClientMessage::AddressInput(s, is_edit) => {
                if is_edit {
                    self.client_to_edit.address = s;
                } else {
                    self.client_to_add.address = s;
                }
            }
            ClientMessage::EmailInput(s, is_edit) => {
                if is_edit {
                    if s == "".to_string() {
                        self.client_to_add.email = None;
                    } else {
                        self.client_to_edit.email = Some(s);
                    }
                } else {
                    self.client_to_add.email = Some(s);
                }
            }
            ClientMessage::ShowAddClient => {
                if self.add_client {
                    self.add_client = false;
                } else {
                    self.add_client = true;
                }
            }
            ClientMessage::Submit(is_edit) => {
                if is_edit {
                    self.edit_client = false;
                } else {
                    self.add_client = false;
                }
            }
            ClientMessage::Delete => {
                self.edit_client = false;
            }
        }
    }

    fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_client {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Column::new()
                            .padding(12)
                            .spacing(8)
                            .push(
                                Text::new("Edit Client".to_string())
                                    .size(24)
                                    .horizontal_alignment(Horizontal::Center)
                                    .width(Length::Fill),
                            )
                            .push(text_input_column(
                                "Name",
                                &self.client_to_edit.name,
                                |input| AppMessage::Client(ClientMessage::NameInput(input, true)),
                                Some(AppMessage::Client(ClientMessage::Submit(true))),
                            ))
                            .push(text_input_column(
                                "Address",
                                &self.client_to_edit.address,
                                |input| {
                                    AppMessage::Client(ClientMessage::AddressInput(input, true))
                                },
                                Some(AppMessage::Client(ClientMessage::Submit(true))),
                            ))
                            .push(text_input_column(
                                "Email",
                                &self.client_to_edit.email.clone().unwrap_or("".to_string()),
                                |input| AppMessage::Client(ClientMessage::EmailInput(input, true)),
                                Some(AppMessage::Client(ClientMessage::Submit(true))),
                            ))
                            .push(
                                Row::new()
                                    .push(
                                        Button::new(
                                            Text::new("Submit".to_string())
                                                .horizontal_alignment(Horizontal::Center),
                                        )
                                        .on_press(AppMessage::Client(ClientMessage::Submit(true)))
                                        .style(CustomMainButtonStyle)
                                        .width(Length::Fill),
                                    )
                                    .push(Column::new().width(Length::Fill))
                                    .push(
                                        Button::new(
                                            Text::new("Delete".to_string())
                                                .horizontal_alignment(Horizontal::Center),
                                        )
                                        .on_press(AppMessage::Client(ClientMessage::Delete))
                                        .width(Length::Fill)
                                        .style(iced::theme::Button::Destructive),
                                    ),
                            ),
                    )
                    .into(),
            )
        } else {
            None
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
                .width(Length::Fill)
                .padding([12, 0, 0, 12])
                .align_items(Alignment::Center)
                .push(Text::new("Clients".to_string()).size(24))
                .push(
                    Row::new()
                        .push(add_button(
                            "Add Cient",
                            AppMessage::Client(ClientMessage::ShowAddClient),
                        ))
                        .padding(12),
                )
                .push_maybe(self.create_view())
                .push_maybe(self.edit_view())
                .push(
                    Container::new(table_header(&["Name", "Address", "Email"]).push(
                        Scrollable::new(Column::new().extend(self.clients.iter().map(|client| {
                            Button::new(
                                Container::new(
                                    Row::new()
                                        .padding(10)
                                        .push(table_column(&client.name))
                                        .push(table_column(&client.address))
                                        .push(table_column(
                                            &client.email.clone().unwrap_or("".to_string()),
                                        )),
                                )
                                .style(table_row_style()),
                            )
                            .style(CustomButtonStyle)
                            .on_press(AppMessage::EditClient(client.clone()))
                            .into()
                        }))),
                    ))
                    .style(table_style()),
                )
                .into(),
        )
        .into()
    }

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_client {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Column::new()
                            .spacing(12)
                            .push(
                                Text::new("Add Client".to_string())
                                    .size(24)
                                    .horizontal_alignment(Horizontal::Center)
                                    .width(Length::Fill),
                            )
                            .push(text_input_column(
                                "Name",
                                &self.client_to_add.name,
                                |input| AppMessage::Client(ClientMessage::NameInput(input, false)),
                                None,
                            ))
                            .push(text_input_column(
                                "Address",
                                &self.client_to_add.address,
                                |input| {
                                    AppMessage::Client(ClientMessage::AddressInput(input, false))
                                },
                                None,
                            ))
                            .push(text_input_column(
                                "Email",
                                &self.client_to_add.email.clone().unwrap_or("".to_string()),
                                |input| AppMessage::Client(ClientMessage::EmailInput(input, false)),
                                Some(AppMessage::Client(ClientMessage::Submit(false))),
                            ))
                            .push(
                                Row::new().push(
                                    Button::new("Submit")
                                        .on_press(AppMessage::Client(ClientMessage::Submit(false)))
                                        .style(CustomMainButtonStyle),
                                ),
                            )
                            .padding(24),
                    )
                    .into(),
            )
        } else {
            None
        }
    }
}

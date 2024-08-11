use std::env;

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, Row, Scrollable, Text},
    Alignment, Element, Length,
};
use sqlx::SqlitePool;

use crate::{
    components::{
        add_button, layout, table_column, table_header, table_row_qty_style, table_style,
        text_input_column, CustomButtonStyle, CustomMainButtonStyle,
    },
    error::Errorr,
    AppMessage,
};

#[derive(Default, Clone, Debug)]
pub struct Part {
    pub part_id: i64,
    pub name: String,
    pub units_left: i64,
    pub cost: f64,
    pub total_spent: f64,
    pub total_units_purchased: i64,
}

#[derive(Default, Clone, Debug)]
pub struct PartToAdd {
    pub name: String,
}

#[derive(Default, Clone)]
pub struct PartsState {
    pub parts: Vec<Part>,
    pub part_to_add: PartToAdd,
    add_part: bool,
    pub part_to_edit: Part,
    pub edit_part: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PartsMessage {
    NameInput(String, bool),
    ShowAddPart,
    ShowEditPart,
    Submit(bool),
    Delete,
}

pub async fn get_parts() -> Result<Vec<Part>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let parts = sqlx::query_as!(Part, "SELECT * FROM Part")
        .fetch_all(&pool)
        .await?;

    Ok(parts)
}

impl PartsState {
    pub async fn add_part(part: PartToAdd) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let name = part.name;

        sqlx::query!(
            "
            INSERT INTO Part (name) 
            VALUES (?)
            ",
            name,
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub async fn edit_part(part: Part) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = part.part_id;
        let name = part.name.as_str();

        sqlx::query!(
            "
            UPDATE Part
            SET name = ?
            WHERE part_id = ?
            ",
            name,
            id
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub async fn delete_part(part: Part) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = part.part_id;

        sqlx::query!(
            "
            DELETE FROM Part
            WHERE part_id = ?
            ",
            id
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub fn update(&mut self, message: PartsMessage) {
        match message {
            PartsMessage::NameInput(s, is_edit) => {
                if is_edit {
                    self.part_to_edit.name = s;
                } else {
                    self.part_to_add.name = s;
                }
            }
            PartsMessage::ShowAddPart => {
                if self.add_part {
                    self.add_part = false;
                } else {
                    self.add_part = true;
                }
            }
            PartsMessage::ShowEditPart => {
                if self.edit_part {
                    self.edit_part = false;
                } else {
                    self.edit_part = true;
                }
            }
            PartsMessage::Submit(is_edit) => {
                if is_edit {
                    self.edit_part = false;
                } else {
                    self.add_part = false;
                }
            }
            PartsMessage::Delete => {
                self.edit_part = false;
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
                .spacing(12)
                .push(Text::new("Parts".to_string()).size(24))
                .width(Length::Fill)
                .padding([12, 0, 0, 12])
                .align_items(Alignment::Center)
                .push(
                    Row::new()
                        .push(add_button(
                            "Add Part",
                            AppMessage::Parts(PartsMessage::ShowAddPart),
                        ))
                        .padding(12),
                )
                .push_maybe(self.create_view())
                .push_maybe(self.edit_view())
                .push(
                    Container::new(
                        table_header(&[
                            "Name",
                            "Cost",
                            "Units Left",
                            "Total Spent",
                            "Total Purchased",
                        ])
                        .push(Scrollable::new(Column::new().extend(
                            self.parts.iter().map(|item| {
                                Button::new(
                                    Container::new(
                                        Row::new()
                                            .padding(10)
                                            .push(table_column(&item.name))
                                            .push(table_column(&format!("${:.2}", item.cost)))
                                            .push(table_column(&item.units_left.to_string()))
                                            .push(table_column(&format!(
                                                "${:.2}",
                                                item.total_spent
                                            )))
                                            .push(table_column(
                                                &item.total_units_purchased.to_string(),
                                            )),
                                    )
                                    .style(table_row_qty_style(item.units_left)),
                                )
                                .style(CustomButtonStyle)
                                .on_press(AppMessage::EditPart(item.clone()))
                                .into()
                            }),
                        ))),
                    )
                    .style(table_style()),
                )
                .into(),
        )
        .into()
    }

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_part {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Column::new()
                            .spacing(8)
                            .push(
                                Text::new("Add Part".to_string())
                                    .size(24)
                                    .horizontal_alignment(Horizontal::Center)
                                    .width(Length::Fill),
                            )
                            .push(text_input_column(
                                "Name",
                                &self.part_to_add.name,
                                |input| AppMessage::Parts(PartsMessage::NameInput(input, false)),
                                Some(AppMessage::Parts(PartsMessage::Submit(false))),
                            ))
                            .push(
                                Button::new(
                                    Text::new("Submit".to_string())
                                        .horizontal_alignment(Horizontal::Center),
                                )
                                .on_press(AppMessage::Parts(PartsMessage::Submit(false)))
                                .style(CustomMainButtonStyle)
                                .width(150),
                            ),
                    )
                    .into(),
            )
        } else {
            None
        }
    }

    pub fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_part {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Column::new()
                            .spacing(8)
                            .push(
                                Text::new("Edit Part".to_string())
                                    .size(24)
                                    .horizontal_alignment(Horizontal::Center)
                                    .width(Length::Fill),
                            )
                            .push(text_input_column(
                                "Name",
                                &self.part_to_edit.name,
                                |input| AppMessage::Parts(PartsMessage::NameInput(input, true)),
                                Some(AppMessage::Parts(PartsMessage::Submit(true))),
                            ))
                            .push(
                                Row::new()
                                    .push(
                                        Button::new(
                                            Text::new("Submit".to_string())
                                                .horizontal_alignment(Horizontal::Center),
                                        )
                                        .on_press(AppMessage::Parts(PartsMessage::Submit(true)))
                                        .style(CustomMainButtonStyle)
                                        .width(Length::Fill),
                                    )
                                    .push(Column::new().width(Length::Fill))
                                    .push(
                                        Button::new(
                                            Text::new("Delete".to_string())
                                                .horizontal_alignment(Horizontal::Center),
                                        )
                                        .on_press(AppMessage::Parts(PartsMessage::Delete))
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
}

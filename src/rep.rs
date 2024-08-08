use std::env;

use iced::{alignment::Horizontal, widget::{Button, Column, Container, Row, Scrollable, Text}, Alignment, Element, Length};
use sqlx::SqlitePool;

use crate::{
    components::{add_button, layout, table_column, table_header, table_row_style, table_style, text_input_column, CustomButtonStyle, CustomMainButtonStyle},
    error::Errorr, AppMessage
};

#[derive(Default, Clone, Debug)]
pub struct Rep {
    pub id: i64,
    pub name: String,
    pub percentage: u8
}

#[derive(Default, Clone)]
pub struct RepState {
    pub reps: Vec<Rep>,
    add_rep: bool,
    pub rep_to_add: Rep,
    pub edit_rep: bool,
    pub rep_to_edit: Rep
}

#[derive(Clone, Debug)]
pub enum RepMessage {
    NameInput(String, bool),
    PercentageInput(String, bool),
    Submit(bool),
    ShowAddRep,
    Delete
}

pub async fn get_reps() -> Result<Vec<Rep>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let reps = sqlx::query_as!(Rep,
                                "SELECT id, name, percentage as `percentage: u8` FROM Rep"
                               )
        .fetch_all(&pool)
        .await?;

    Ok(reps)
}

pub async fn get_rep(i: i64) -> Result<Rep, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let rep = sqlx::query_as!(Rep,
                                "SELECT id, name, percentage as `percentage: u8` FROM Rep WHERE id = ?",
                                i
                               )
        .fetch_one(&pool)
        .await?;

    Ok(rep)
}

pub async fn add_rep(rep: Rep) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let name = rep.name;
    let percentage = rep.percentage;

    sqlx::query!(
        "
        INSERT INTO Rep (name, percentage)
        VALUES (?,?)
        ",
        name,
        percentage
        )
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn edit_rep(rep: Rep) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let id = rep.id;
    let name = rep.name;
    let percentage = rep.percentage;

    sqlx::query!(
        "
        UPDATE Rep 
        SET name = ?, percentage = ?
        WHERE id = ?
        ",
        name,
        percentage,
        id
        )
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn delete_rep(rep: Rep) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let id = rep.id;

    sqlx::query!(
        "
        DELETE FROM Rep
        WHERE id = ?
        ",
        id
        )
        .execute(&pool)
        .await?;

    Ok(())
}

impl RepState {
    pub fn update(&mut self, message: RepMessage) {
       match message {
           RepMessage::NameInput(s, is_edit) => {
               if is_edit {
                   self.rep_to_edit.name = s;
               } else {
                   self.rep_to_add.name = s;
               }
           }
           RepMessage::PercentageInput(s, is_edit) => {
               if is_edit {
                   self.rep_to_edit.percentage = s.parse::<u8>().unwrap_or(0);
               } else {
                   self.rep_to_add.percentage = s.parse::<u8>().unwrap_or(0);
               }
           }
           RepMessage::ShowAddRep => {
               if self.add_rep {
                   self.add_rep = false;
               } else {
                   self.add_rep = true;
               }
           }
           RepMessage::Submit(is_edit) => {
               if is_edit {
                   self.edit_rep = false;
               } else {
                   self.add_rep = false;
               }
           }
           RepMessage::Delete => {
               self.edit_rep = false;
           }
       }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
            .width(Length::Fill)
            .padding([12, 0, 0, 12])
            .align_items(Alignment::Center)
            .push(Text::new("Reps".to_string())
                  .size(24))
            .push(Row::new()
                  .push(
                    add_button("Add Rep", AppMessage::Rep(RepMessage::ShowAddRep))
                    )
                  .padding(12))
                   .push_maybe(self.create_view())
                  .push_maybe(self.edit_view())
            .push(Container::new(
                    table_header(&["Name", "Percentage"])
                    .push(Scrollable::new(
                            Column::new()
                            .extend(self.reps.iter().map(
                                    |rep| 
                                    Button::new(
                                        Container::new(
                                            Row::new()
                                            .padding(10)
                                            .push(table_column(&rep.name))
                                            .push(table_column(&format!("{}%", &rep.percentage))))
                                        .style(table_row_style())
                                        )
                                    .style(CustomButtonStyle)
                                    .on_press(AppMessage::EditRep(rep.clone()))
                                    .into()
                                    ))))
                    )
                    .style(table_style())).into())
                            .into()
    }
    
    fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_rep {
            Some(
                Column::new()
                .max_width(1000)
                .push(
                    Column::new()
                    .padding(12)
                    .spacing(8)
                    .push(
                        Text::new("Edit Rep".to_string())
                        .size(24)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                        )
                    .push(
                        text_input_column(
                            "Name", 
                            &self.rep_to_edit.name, 
                            |input| {
                            AppMessage::Rep(
                                RepMessage::NameInput(input, true),
                                )
                        },
                        Some(AppMessage::Rep(RepMessage::Submit(true)))
                        )
                        )
                    .push(
                        text_input_column(
                            "Percentage", 
                            &self.rep_to_edit.percentage.to_string(), 
                            |input| {
                            AppMessage::Rep(
                                RepMessage::PercentageInput(input, true),
                                )
                        },
                        Some(AppMessage::Rep(RepMessage::Submit(true)))
                        )
                        )
                    .push(
                        Row::new()
                        .push(
                            Button::new(
                                Text::new("Submit".to_string())
                                .horizontal_alignment(Horizontal::Center),
                                )
                            .on_press(AppMessage::Rep(RepMessage::Submit(true)))
                            .style(CustomMainButtonStyle)
                            .width(Length::Fill),
                            )
                        .push(Column::new().width(Length::Fill))
                        .push(
                            Button::new(
                                Text::new("Delete".to_string())
                                .horizontal_alignment(Horizontal::Center),
                                )
                            .on_press(AppMessage::Rep(RepMessage::Delete))
                            .width(Length::Fill)
                            .style(iced::theme::Button::Destructive),
                            ),
                            )
                                )
                                .into(),
                                )
        } else {
            None
        }
    }

    fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_rep {
            Some(
                Column::new()
                .max_width(1000)
                .push(
                    Column::new()
                    .spacing(12)
                    .push(
                        Text::new("Add Rep".to_string())
                        .size(24)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                        )
                    .push(
                        text_input_column(
                            "Name",
                            &self.rep_to_add.name,
                            |input| { 
                                AppMessage::Rep(RepMessage::NameInput(input, false))
                            },
                            None
                            )
                        )
                    .push(
                        text_input_column(
                            "Percentage",
                            &self.rep_to_add.percentage.to_string(),
                            |input| {
                                AppMessage::Rep(RepMessage::PercentageInput(input, false))
                            },
                            Some(AppMessage::Rep(RepMessage::Submit(false)))
                            )
                        )
                    .push(
                        Row::new()
                        .push(
                            Button::new("Submit")
                            .on_press(AppMessage::Rep(
                                    RepMessage::Submit(false),
                                    ))
                            .style(CustomMainButtonStyle)
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

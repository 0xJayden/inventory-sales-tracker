use std::env;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Button, Column, Container, Row, Scrollable, Text, TextInput},
    Alignment, Element, Length,
};
use regex::Regex;
use sqlx::SqlitePool;

use crate::{
    components::{
        add_button, bold_text, card_style, close_button, close_edit_row, layout, table_column, table_header, table_row_style, table_style, text_input_column, CustomButtonStyle, CustomMainButtonStyle
    },
    error::Errorr,
    parts::Part,
    AppMessage,
};

#[derive(Default, Clone, Debug)]
pub struct Purchase {
    pub id: i64,
    pub date: String,
    pub total: f64,
    pub note: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct PurchasePart {
    pub id: i64,
    name: String,
    qty: i64,
    cost: f64,
}

#[derive(Default, Clone, Debug)]
pub struct PurchaseToAdd {
    pub date: String,
    pub total: f64,
    pub note: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct PartToSelect {
    pub part_id: i64,
    pub name: String,
    pub cost: String,
    pub qty: i64,
    pub total_spent: f64,
    pub total_units_purchased: i64,
}

#[derive(Default, Clone, Debug)]
pub struct PartToCreate {
    pub name: String,
}

#[derive(Default, Clone)]
pub struct PurchaseState {
    pub purchases: Vec<Purchase>,
    pub purchase_to_add: PurchaseToAdd,
    add_purchase: bool,
    pub purchase_to_edit: Purchase,
    pub edit_purchase: bool,
    pub parts: Vec<Part>,
    pub parts_to_select: Vec<PartToSelect>,
    pub parts_to_add: Vec<PartToSelect>,
    pub create_part: bool,
    pub part_to_create: PartToCreate,
    pub view_purchase: bool,
    pub purchase_to_view: Purchase,
    pub purchase_parts_to_view: Vec<PurchasePart>,
    query: String,
    pub filtered_parts: Vec<PartToSelect>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PurchaseMessage {
    DateInput(String, bool),
    NoteInput(String, bool),
    ShowAddPurchase,
    PartQtyChanged(String, i64),
    PartCostChanged(String, i64),
    RemovePart(i64),
    CreatePart,
    PartName(String),
    CreatePartSubmit,
    Submit(bool),
    Delete,
    Query(String),
    CloseView,
}

fn select_part_header() -> Container<'static, AppMessage> {
    Container::new(
        Row::new()
            .width(Length::Fill)
            .padding(8)
            .push(Column::new().push(bold_text("Name")).width(120))
            .push(Column::new().push(bold_text("Cost")).width(110))
            .push(Column::new().push(bold_text("Qty")).width(60)),
    )
    .style(table_row_style())
}

pub async fn get_purchases() -> Result<Vec<Purchase>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let purchases = sqlx::query_as!(Purchase, "SELECT * FROM Purchase")
        .fetch_all(&pool)
        .await?;

    Ok(purchases)
}

pub async fn get_purchase_parts(purchase_id: i64) -> Result<Vec<PurchasePart>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let purchases = sqlx::query_as!(
        PurchasePart,
        "SELECT PurchasePart.qty, PurchasePart.cost, PurchasePart.id,
                                   Part.name as name
                                   FROM PurchasePart
                                   JOIN Part ON PurchasePart.part_id = Part.part_id
                                   WHERE PurchasePart.purchase_id = ?
                                   ",
        purchase_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(purchases)
}

pub async fn delete_purchase(purchase: Purchase) -> Result<(), Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let id = purchase.id;

    sqlx::query!(
        "
        DELETE FROM Purchase
        WHERE id = ?
        ",
        id
        )
        .execute(&pool)
        .await?;

    Ok(())
}

fn part_view_row(label: &str, value: String) -> Row<'static, AppMessage> {
    Row::new()
        .padding(4)
        .push(Row::new().push(Text::new(label.to_string())).width(100))
        .push(
            Column::new()
                .push(Text::new(value))
                .align_items(Alignment::End)
                .width(100),
        )
}

fn part_view(part: &PurchasePart) -> Container<'static, AppMessage> {
    Container::new(
        Column::new()
            .push(Text::new(part.name.to_string()).size(20))
            .push(part_view_row("Name: ", part.name.clone()))
            .push(part_view_row("Quantity: ", part.qty.to_string()))
            .push(part_view_row("Cost: ", format!("${:.2}", part.cost))),
    )
    .padding(8)
    .style(card_style())
}

pub fn parse_input(s: &str) -> &str {
    match s {
        "0" => "",
        _ => s,
    }
}

pub fn validate_input(s: &str) -> bool {
    let re = Regex::new(r"^[0-9.]*$").unwrap();
    if re.is_match(s) {
        true
    } else {
        println!("invalid input: {}", s);
        false
    }
}

impl PurchaseState {
    pub async fn add_purchase(
        parts_to_add: Vec<PartToSelect>,
        purchase_to_add: PurchaseToAdd,
        parts: Vec<Part>,
    ) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let date = purchase_to_add.date;
        let total = purchase_to_add.total;
        let note = purchase_to_add.note;

        let r = sqlx::query!(
            "
            INSERT INTO Purchase (total, date, note)
            VALUES (?,?,?)
            ",
            total,
            date,
            note
        )
        .execute(&pool)
        .await?;

        let purchase_id = r.last_insert_rowid();

        for part in &parts_to_add {
            sqlx::query!(
                "
                INSERT INTO PurchasePart (qty, cost, purchase_id, part_id)
                VALUES (?,?,?,?)
                ",
                part.qty,
                part.cost,
                purchase_id,
                part.part_id
            )
            .execute(&pool)
            .await?;

            let partt = parts.iter().find(|p| p.part_id == part.part_id).unwrap();
            let total_units = partt.total_units_purchased + part.qty;
            let total_spent = partt.total_spent + part.cost.parse::<f64>().unwrap_or(0.00);
            let units_left = partt.units_left + part.qty;
            let cost = total_spent / total_units as f64;

            sqlx::query!(
                "
                UPDATE Part
                SET total_units_purchased = ?, total_spent = ?, units_left = ?, cost = ? 
                WHERE part_id = ?
                ",
                total_units,
                total_spent,
                units_left,
                cost,
                part.part_id
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "
                UPDATE ProductPart
                SET cost = ?
                WHERE part_id = ?
                ",
                cost,
                part.part_id
            )
            .execute(&pool)
            .await?;
        }

        let products = sqlx::query!(
            "
            SELECT * FROM Product
            "
        )
        .fetch_all(&pool)
        .await?;

        for product in &products {
            let product_parts = sqlx::query!(
                "
                SELECT * FROM ProductPart
                WHERE product_id = ?  
                ",
                product.product_id
            )
            .fetch_all(&pool)
            .await?;

            let mut cost = 0.00;
            for pp in &product_parts {
                cost += pp.cost * pp.qty as f64;
            }

            sqlx::query!(
                "
                UPDATE Product
                SET cost = ? 
                WHERE product_id = ?
                ",
                cost,
                product.product_id
            )
            .execute(&pool)
            .await?;
        }

        Ok(())
    }

    pub fn update(&mut self, message: PurchaseMessage) {
        match message {
            PurchaseMessage::DateInput(d, is_edit) => {
                if is_edit {
                    self.purchase_to_edit.date = d;
                } else {
                    self.purchase_to_add.date = d;
                }
            }
            PurchaseMessage::NoteInput(n, is_edit) => {
                if is_edit {
                    self.purchase_to_edit.note = Some(n);
                } else {
                    self.purchase_to_add.note = Some(n);
                }
            }
            PurchaseMessage::ShowAddPurchase => {
                if self.add_purchase {
                    self.add_purchase = false;
                } else {
                    self.view_purchase = false;
                    self.add_purchase = true;
                }
            }
            PurchaseMessage::PartQtyChanged(q, id) => {
                if let Some(i) = self
                    .filtered_parts
                    .iter_mut()
                    .find(|item| item.part_id == id)
                {
                    i.qty = q.parse::<i64>().unwrap_or(0);
                    match self.parts_to_add.iter_mut().find(|item| item.part_id == id) {
                        Some(p) => {
                            p.qty = i.qty;
                            if p.qty == 0 && p.cost == "" {
                                let f_parts =
                                    self.parts_to_add.iter().filter_map(|part| {
                                        match part.part_id != id {
                                            true => Some(part.to_owned()),
                                            false => None,
                                        }
                                    });

                                self.parts_to_add = f_parts.collect();
                            }
                        }
                        None => self.parts_to_add.push(i.clone()),
                    }
                }
            }
            PurchaseMessage::PartCostChanged(q, id) => {
                if let Some(i) = self
                    .filtered_parts
                    .iter_mut()
                    .find(|item| item.part_id == id)
                {
                    let valid_input = validate_input(&q);
                    if valid_input {
                        i.cost = q;
                    }

                    match self.parts_to_add.iter_mut().find(|item| item.part_id == id) {
                        Some(p) => {
                            p.cost = i.cost.clone();
                            if p.qty == 0 && p.cost == "" {
                                let f_parts =
                                    self.parts_to_add.iter().filter_map(|part| {
                                        match part.part_id != id {
                                            true => Some(part.to_owned()),
                                            false => None,
                                        }
                                    });

                                self.parts_to_add = f_parts.collect();
                            }
                        }
                        None => self.parts_to_add.push(i.clone()),
                    }
                }
            }
            PurchaseMessage::RemovePart(id) => {
                let f_parts =
                    self.parts_to_add
                        .iter()
                        .filter_map(|part| match part.part_id != id {
                            true => Some(part.to_owned()),
                            false => None,
                        });

                self.parts_to_add = f_parts.collect();
            }
            PurchaseMessage::CreatePart => {
                if self.create_part {
                    self.create_part = false;
                } else {
                    self.create_part = true;
                }
            }
            PurchaseMessage::PartName(s) => {
                self.part_to_create.name = s;
            }
            PurchaseMessage::CreatePartSubmit => {
                self.create_part = false;
            }
            PurchaseMessage::Submit(is_edit) => {
                if is_edit {
                    self.edit_purchase = false;
                } else {
                    for x in &self.parts_to_add {
                        self.purchase_to_add.total += x.cost.parse::<f64>().unwrap_or(0.00);
                    }
                    self.add_purchase = false;
                }
            }
            PurchaseMessage::Delete => {
                self.edit_purchase = false;
                self.view_purchase = false;
            }
            PurchaseMessage::Query(q) => {
                if q.len() > 0 {
                    self.filtered_parts = self
                        .parts_to_select
                        .iter()
                        .filter_map(|part| {
                            if part.name.contains(&q) {
                                Some(part.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect();
                } else {
                    self.filtered_parts = self.parts_to_select.clone()
                }
                self.query = q;
            }
            PurchaseMessage::CloseView => {
                self.view_purchase = false;
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
                .width(Length::Fill)
                .padding([12, 0, 0, 12])
                .align_items(Alignment::Center)
                .push(Text::new("Purchases".to_string()).size(24))
                .push(
                    Row::new()
                        .push(add_button(
                            "Add Purchase",
                            AppMessage::Purchase(PurchaseMessage::ShowAddPurchase),
                        ))
                        .padding(12),
                )
                .push_maybe(self.view_purchase())
                .push_maybe(self.create_view())
                .push_maybe(self.edit_view())
                .push(
                    Container::new(table_header(&["Date", "Total", "Note"]).push(
                        Scrollable::new(Column::new().padding([0, 8, 0, 0]).extend(
                            self.purchases.iter().map(|purchase| {
                                Button::new(
                                    Container::new(
                                        Row::new()
                                            .padding(10)
                                            .push(table_column(&purchase.date))
                                            .push(table_column(&format!("${:.2}", purchase.total)))
                                            .push(table_column(
                                                &purchase.note.clone().unwrap_or("".to_string()),
                                            )),
                                    )
                                    .style(table_row_style()),
                                )
                                .style(CustomButtonStyle)
                                .on_press(AppMessage::ViewPurchase(purchase.clone()))
                                .into()
                            }),
                        )),
                    ))
                    .style(table_style()),
                )
                .into(),
        )
        .into()
    }

    fn select_part(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .spacing(8)
                .push(
                    Row::new()
                        .push(
                            Column::new()
                                .width(Length::Fill)
                                .push(bold_text("Add Parts")),
                        )
                        .push(
                            Column::new()
                                .width(Length::Fill)
                                .align_items(Alignment::End)
                                .push(
                                    Button::new("Create Part")
                                        .on_press(AppMessage::Purchase(PurchaseMessage::CreatePart))
                                        .style(CustomMainButtonStyle),
                                ),
                        ),
                )
                .push_maybe(self.create_part_view())
                .push(
                    TextInput::new("Search", &self.query)
                        .on_input(|input| AppMessage::Purchase(PurchaseMessage::Query(input))),
                )
                .push(Container::new(
                    Column::new()
                        .spacing(4)
                        .width(Length::Fill)
                        .push(select_part_header())
                        .push(Scrollable::new(Column::new().spacing(4).extend(
                            self.filtered_parts.iter().map(|part| {
                                Container::new(
                                    Row::new()
                                        .width(Length::Fill)
                                        .spacing(4)
                                        .padding(8)
                                        .push(
                                            Container::new(Text::new(&part.name))
                                                .align_y(Vertical::Center)
                                                .height(32)
                                                .width(120),
                                        )
                                        .push(
                                            TextInput::new("Cost", parse_input(&part.cost))
                                                .width(100)
                                                .on_input(|input| {
                                                    AppMessage::Purchase(
                                                        PurchaseMessage::PartCostChanged(
                                                            input,
                                                            part.part_id,
                                                        ),
                                                    )
                                                }),
                                        )
                                        .push(
                                            TextInput::new("Quantity", &part.qty.to_string())
                                                .width(50)
                                                .on_input(|input| {
                                                    AppMessage::Purchase(
                                                        PurchaseMessage::PartQtyChanged(
                                                            input,
                                                            part.part_id,
                                                        ),
                                                    )
                                                }),
                                        ),
                                )
                                .align_y(Vertical::Center)
                                .style(table_row_style())
                                .into()
                            }),
                        ))),
                )),
        )
        .max_height(300)
    }

    fn selected_parts(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .spacing(8)
                .push(Row::new().push(bold_text("Selected Parts")))
                .push(
                    Column::new()
                        .spacing(4)
                        .width(Length::Fill)
                        .push(select_part_header())
                        .push(Scrollable::new(Column::new().spacing(4).extend(
                            self.parts_to_add.iter().map(|part| {
                                Container::new(
                                    Row::new()
                                        .width(Length::Fill)
                                        .spacing(4)
                                        .padding(8)
                                        .push(
                                            Container::new(Text::new(&part.name))
                                                .align_y(Vertical::Center)
                                                .height(32)
                                                .width(120),
                                        )
                                        .push(
                                            Container::new(Text::new(format!(
                                                "${:.2}",
                                                &part.cost.parse::<f64>().unwrap_or(0.00)
                                            )))
                                            .align_y(Vertical::Center)
                                            .height(32)
                                            .width(100),
                                        )
                                        .push(
                                            Container::new(Text::new(part.qty.to_string()))
                                                .align_y(Vertical::Center)
                                                .height(32)
                                                .width(50),
                                        )
                                        .push(close_button(AppMessage::Purchase(
                                            PurchaseMessage::RemovePart(part.part_id),
                                        ))),
                                )
                                .style(table_row_style())
                                .into()
                            }),
                        ))),
                ),
        )
        .max_height(250)
    }

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_purchase {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Container::new(
                            Column::new()
                                .spacing(12)
                                .push(
                                    Text::new("Add Purchase".to_string())
                                        .size(24)
                                        .horizontal_alignment(Horizontal::Center)
                                        .width(Length::Fill),
                                )
                                .push(text_input_column(
                                    "Date",
                                    &self.purchase_to_add.date,
                                    |input| {
                                        AppMessage::Purchase(PurchaseMessage::DateInput(
                                            input, false,
                                        ))
                                    },
                                    None,
                                ))
                                .push(text_input_column(
                                    "Note",
                                    &self.purchase_to_add.note.clone().unwrap_or("".to_string()),
                                    |input| {
                                        AppMessage::Purchase(PurchaseMessage::NoteInput(
                                            input, false,
                                        ))
                                    },
                                    None,
                                ))
                                .push(
                                    Column::new()
                                        .width(Length::Fill)
                                        .align_items(Alignment::Center)
                                        .push(
                                            Row::new()
                                                .spacing(12)
                                                .push(self.select_part())
                                                .push(self.selected_parts()),
                                        ),
                                )
                                .push(
                                    Row::new().push(
                                        Button::new("Submit")
                                            .on_press(AppMessage::Purchase(
                                                PurchaseMessage::Submit(false),
                                            ))
                                            .style(CustomMainButtonStyle),
                                    ),
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

    pub fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_purchase {
            Some(
                Container::new(
                    Column::new()
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                        .push(
                            Column::new()
                                .max_width(700)
                                .push(
                                    Text::new("Edit Purchase".to_string())
                                        .size(24)
                                        .horizontal_alignment(Horizontal::Center)
                                        .width(Length::Fill),
                                )
                                .push(text_input_column(
                                    "Date",
                                    &self.purchase_to_edit.date,
                                    |input| {
                                        AppMessage::Purchase(PurchaseMessage::DateInput(
                                            input, true,
                                        ))
                                    },
                                    Some(AppMessage::Purchase(PurchaseMessage::Submit(true))),
                                ))
                                .push(
                                    Row::new()
                                        .push(
                                            Button::new(
                                                Text::new("Submit".to_string())
                                                    .horizontal_alignment(Horizontal::Center),
                                            )
                                            .on_press(AppMessage::Purchase(
                                                PurchaseMessage::Submit(true),
                                            ))
                                            .style(CustomMainButtonStyle)
                                            .width(Length::Fill),
                                        )
                                        .push(Column::new().width(Length::Fill))
                                        .push(
                                            Button::new(
                                                Text::new("Delete".to_string())
                                                    .horizontal_alignment(Horizontal::Center),
                                            )
                                            .on_press(AppMessage::Purchase(PurchaseMessage::Delete))
                                            .width(Length::Fill)
                                            .style(iced::theme::Button::Destructive),
                                        ),
                                )
                                .padding(24),
                        ),
                )
                .into(),
            )
        } else {
            None
        }
    }

    fn create_part_view(&self) -> Option<Element<AppMessage>> {
        if self.create_part {
            Some(
                Container::new(Scrollable::new(
                    Column::new()
                        .push(text_input_column(
                            "Name",
                            &self.part_to_create.name,
                            |input| AppMessage::Purchase(PurchaseMessage::PartName(input)),
                            Some(AppMessage::Purchase(PurchaseMessage::CreatePartSubmit)),
                        ))
                        .push(
                            Button::new("Submit")
                                .on_press(AppMessage::Purchase(PurchaseMessage::CreatePartSubmit))
                                .style(CustomMainButtonStyle),
                        ),
                ))
                .into(),
            )
        } else {
            None
        }
    }

    fn view_purchase(&self) -> Option<Element<AppMessage>> {
        if self.view_purchase {
            Some(
                Container::new(
                    Column::new()
                    .max_width(300)
                    .push(
                        close_edit_row(
                            AppMessage::Purchase(PurchaseMessage::CloseView), 
                            AppMessage::EditPurchase(self.purchase_to_view.clone())
                            )
                        )
                    .push(Row::new().push(Text::new("Purchase")))
                    .push(Row::new().push(Text::new(&self.purchase_to_view.date)))
                    .push(Row::new().push(Text::new("Parts")))
                    .push(
                        Column::new().spacing(8).extend(
                            self.purchase_parts_to_view
                            .iter()
                            .map(|part| Row::new().push(part_view(&part)).into()),
                            ),
                            )
                    .padding([0, 12, 0, 0]),
                    )
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                .into(),
            )
        } else {
            None
        }
    }
}

use std::env;

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, Row, Scrollable, Text, TextInput},
    Alignment, Element, Length,
};
use sqlx::SqlitePool;

use crate::{
    components::{add_button, bold_text, card_style, close_button, edit_column, layout, table_column, table_header, table_row_qty_style, table_row_style, table_style, text_input_column, CustomButtonStyle, CustomMainButtonStyle}, error::Errorr, manufacture::select_header, parts::Part, purchase::{parse_input, validate_input, PartToSelect}, AppMessage
};

#[derive(Debug, Default, Clone)]
pub struct Product {
    pub product_id: i64,
    pub name: String,
    pub units: i64,
    pub cost: f64,
    pub msrp: f64,
}

#[derive(Debug, Default, Clone)]
pub struct ProductToAdd {
    pub name: String,
    pub msrp: String,
}

#[derive(Debug, Default, Clone)]
pub struct ProductPart {
    pub product_part_id: i64,
    pub name: String,
    pub qty: i64,
    pub cost: f64,
    pub part_id: i64,
    pub product_id: i64
}

#[derive(Default, Clone)]
pub struct ProductState {
    pub products: Vec<Product>,
    pub product_to_add: ProductToAdd,
    pub product_to_edit: Product,
    show_add_product: bool,
    pub edit_product: bool,
    pub parts: Vec<Part>,
    pub parts_to_add: Vec<PartToSelect>,
    pub parts_to_select: Vec<PartToSelect>,
    create_part: bool,
    part_to_create: Part,
    pub product_to_view: Product,
    pub view_product: bool,
    pub product_parts_to_view: Vec<ProductPart>,
    query: String,
    pub filtered_parts: Vec<PartToSelect>
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProductMessage {
    NameInput(String, bool),
    MsrpInput(String, bool),
    Submit(bool),
    ShowAddProduct,
    Delete,
    PartName(String),
    CreatePartSubmit,
    CreatePart,
    PartQtyChanged(String, i64),
    RemovePart(i64),
    Query(String),
    CloseView
}

pub async fn get_products() -> Result<Vec<Product>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let products = sqlx::query_as!(Product, "SELECT * FROM Product")
        .fetch_all(&pool)
        .await?;

    Ok(products)
}

pub async fn get_product_parts(product_id: i64) -> Result<Vec<ProductPart>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let products = sqlx::query_as!(ProductPart,
                                   "SELECT ProductPart.product_part_id, ProductPart.qty, ProductPart.part_id, ProductPart.product_id, 
                                   Part.name as name, Part.cost as cost
                                   FROM ProductPart
                                   JOIN Part ON ProductPart.part_id = Part.part_id
                                   WHERE ProductPart.product_id = ?
                                   ",
                                   product_id
                                   )
        .fetch_all(&pool)
        .await?;

    Ok(products)
}

fn part_view_row(label: &str, value: String) -> Row<'static, AppMessage> {
    Row::new()
        .padding(4)
        .push(Row::new()
              .push(Text::new(label.to_string()))
              .width(100))
        .push(Column::new()
              .push(Text::new(value))
              .align_items(Alignment::End)
              .width(100))
}

fn part_view(part: &ProductPart) -> Container<'static, AppMessage> {
    Container::new(
        Column::new()
        .push(Text::new(part.name.to_string())
              .size(20))
        .push(part_view_row("Name: ", part.name.clone()))
        .push(part_view_row("Quantity: ", part.qty.to_string()))
        .push(part_view_row("Cost: ", format!("${:.2}", part.cost)))
        .push(part_view_row("Total: ", format!("${:.2}", part.cost * part.qty as f64))))
        .padding(8)
        .style(card_style())
}

impl ProductState {
    pub async fn add_product(product: ProductToAdd,  parts_to_add: Vec<PartToSelect>) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let name = product.name;
        let msrp = product.msrp;
        let mut cost = 0.00;

        for part in &parts_to_add {
            cost += part.cost.parse::<f64>().unwrap_or(0.00) * part.qty as f64;
        };

        let r = sqlx::query!(
            "
            INSERT INTO Product (name, msrp, cost)
            VALUES (?,?,?)
            ",
            name,
            msrp,
            cost
        )
        .execute(&pool)
        .await?;

        let product_id = r.last_insert_rowid();

        for part in &parts_to_add {
            sqlx::query!(
                "
                INSERT INTO ProductPart (qty, cost, product_id, part_id)
                VALUES (?,?,?,?)
                ",
                part.qty,
                part.cost,
                product_id,
                part.part_id
            )
            .execute(&pool)
            .await?;
        }
        
        Ok(())
    }

    pub async fn edit_product(product: Product) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = product.product_id;
        let name = product.name.as_str();
        let units = product.units;
        let cost = product.cost;
        let msrp = product.msrp;

        sqlx::query!(
            "
            UPDATE Product
            SET name = ?, units = ?, cost = ?, msrp = ?
            WHERE product_id = ?
            ",
            name,
            units,
            cost,
            msrp,
            id
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    pub async fn delete_product(product: Product) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = product.product_id;

        sqlx::query!(
            "
            DELETE FROM Product
            WHERE product_id = ?
            ",
            id
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    fn select_part(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(4)
                .push(
                    Row::new()
                    .width(Length::Fill)
                    .push(
                        Column::new()
                        .push(bold_text("Add Parts"))
                        .width(Length::Fill)
                        )
                    .push(
                        Column::new()
                        .width(Length::Fill)
                        .align_items(Alignment::End)
                        .push(
                            Button::new("Create Part")
                            .on_press(AppMessage::Product(ProductMessage::CreatePart))
                            .style(CustomMainButtonStyle)
                            )),
                            )
                .push_maybe(self.create_part_view())
                .push(
                    TextInput::new("Search", &self.query)
                    .on_input(|input| AppMessage::Product(ProductMessage::Query(input)))
                    )
                .push(
                    Container::new(
                        Column::new()
                        .width(Length::Fill)
                        .push(select_header())
                        .push(
                            Scrollable::new(
                                Column::new()
                                .spacing(4)
                                .extend(
                                    self.filtered_parts
                                    .iter()
                                    .map(|part| {
                                        Container::new(
                                        Row::new()
                                            .width(Length::Fill)
                                            .spacing(4)
                                            .padding(8)
                                            .push(
                                                Column::new()
                                                .push(
                                                    Row::new()
                                                      .push(table_column(&part.name))),
                                                )
                                            .push(
                                                TextInput::new("Quantity", &part.qty.to_string())
                                                .width(50)
                                                .on_input(|input| {
                                                    AppMessage::Product(ProductMessage::PartQtyChanged(
                                                            input,
                                                            part.part_id,
                                                            ))
                                                }),
                                                ))
                                            .style(table_row_style())
                                            .into()
                                    }),
                                    ))),
                        )
        ))
        .max_height(300)
    }

    fn selected_parts(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .width(Length::Fill)
            .push(
                Row::new()
                .push(bold_text("Selected Parts")))
            .push(
                Container::new(
                    Column::new()
                    .width(Length::Fill)
                    .push(select_header())
                    .push(
                        Scrollable::new(
                            Column::new()
                            .spacing(4)
                            .extend(
                                self.parts_to_add
                                .iter()
                                .map(|part| {
                                    Container::new(
                                        Row::new()
                                        .width(Length::Fill)
                                        .spacing(4)
                                        .padding(8)
                                        .push(
                                            Column::new()
                                            .width(100)
                                            .align_items(Alignment::Center)
                                            .push(Text::new(&part.name)),
                                            )
                                        .push(
                                            Column::new()
                                            .width(50)
                                            .align_items(Alignment::Center)
                                            .push(
                                                Text::new(part.qty.to_string())
                                                )
                                            )
                                        .push(
                                            close_button(AppMessage::Product(ProductMessage::RemovePart(part.part_id)))
                                            ))
                                            .style(table_row_style())
                                            .into()
                                }),
                                ))))
                                    ),
                                    )
                                        .max_height(300)
    }

    pub fn update(&mut self, message: ProductMessage) {
        match message {
            ProductMessage::NameInput(input, is_edit) => {
                if is_edit {
                    self.product_to_edit.name = input;
                } else {
                    self.product_to_add.name = input;
                }
            }
            ProductMessage::MsrpInput(input, is_edit) => {
                if is_edit {
                    self.product_to_edit.msrp = input.parse::<f64>().unwrap_or(0.00);
                } else {
                    let valid_input = validate_input(&input);
                    if valid_input {
                        self.product_to_add.msrp = input;
                    }
                }
            }
            ProductMessage::Submit(is_edit) => {
                if is_edit {
                    self.edit_product = false;
                } else {
                    self.show_add_product = false;
                }
            }
            ProductMessage::ShowAddProduct => {
                if self.show_add_product {
                    self.show_add_product = false;
                } else {
                    self.view_product = false;
                    self.show_add_product = true;
                }
            }
            ProductMessage::Delete => {
                println!("delete");
            }
            ProductMessage::PartName(s) => {
                self.part_to_create.name = s;
            }
            ProductMessage::PartQtyChanged(q, id) => {
                if let Some(i) = self.filtered_parts.iter_mut().find(|item| item.part_id == id) {
                    i.qty = q.parse::<i64>().unwrap_or(0);
                    match self.parts_to_add.iter_mut().find(|item| item.part_id == id) {
                        Some(p) => {
                            p.qty = i.qty;
                            if p.qty == 0 {
                                let f_parts = self.parts_to_add.iter().filter_map(|part| match part.part_id != id {
                                    true => Some(part.to_owned()),
                                    false => None
                                });
                
                                self.parts_to_add = f_parts.collect();
                            }
                        }
                        None => {
                            self.parts_to_add.push(i.clone())
                        }
                    }
                }
            }
            ProductMessage::RemovePart(id) => {
                let f_parts = self.parts_to_add.iter().filter_map(|part| match part.part_id != id {
                    true => Some(part.to_owned()),
                    false => None
                });
                
                self.parts_to_add = f_parts.collect();
            }
            ProductMessage::CreatePartSubmit => {
                println!("fdsjfl");
            }
            ProductMessage::CreatePart => {
               if self.create_part {
                   self.create_part = false;
               } else {
                   self.create_part = true;
               }
            }
            ProductMessage::Query(q) => {
                if q.len() > 0 {
                    self.filtered_parts = self.parts_to_select.iter().filter_map(|part| if part.name.contains(&q) {
                        Some(part.to_owned())
                    } else {
                        None
                    }
                    ).collect();
                } else {
                    self.filtered_parts = self.parts_to_select.clone()
                }
                self.query = q;
            }
            ProductMessage::CloseView => {
                self.view_product = false;
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
            .spacing(12)
            .padding(24)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(Text::new("Products".to_string()).size(24))
            .push(
                Row::new()
                .push(
                    add_button("Add Product", AppMessage::Product(ProductMessage::ShowAddProduct))
                    )
                .padding(12),
                )
            .push_maybe(self.create_view())
            .push_maybe(self.edit_view())
            .push_maybe(self.view_product())
            .push(
                Container::new(
                    table_header(&["Name", "Units", "Cost", "MSRP", "Net"])
                    .push(
                        Scrollable::new(
                            Column::new()
                            .extend(
                                self.products
                                .iter()
                                .map(|product| {
                                    Button::new(
                                        Container::new(
                                            Row::new()
                                            .padding(10)
                                            .push(table_column(&product.name))
                                            .push(table_column(
                                                    &product.units.to_string().as_str(),
                                                    ))
                                            .push(table_column(
                                                    format!("${:.2}", product.cost).as_str(),
                                                    ))
                                            .push(table_column(
                                                    format!("${:.2}", product.msrp).as_str(),
                                                    ))
                                            .push(table_column(
                                                    format!(
                                                        "${:.2}",
                                                        (product.msrp - product.cost)
                                                        * product
                                                        .units
                                                        .to_string()
                                                        .parse::<f64>()
                                                        .unwrap()
                                                        )
                                                    .as_str(),
                                                    )),
                                                    )
                                                        .style(table_row_qty_style(product.units)),
                                                        )
                                                            .style(CustomButtonStyle)
                                                            .on_press(AppMessage::ViewProduct(product.clone()))
                                                            .into()
                                }),
                                ))),
                                )
                                    .style(table_style()),
                                    ).into()
                                        )
                                        .into()
    }

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.show_add_product {
            Some(
                Column::new()
                .max_width(1000)
                .push(
                    Column::new()
                    .spacing(12)
                    .push(
                        Text::new("Add Product".to_string())
                        .size(24)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::Fill),
                        )
                    .push(
                        text_input_column("Name", &self.product_to_add.name, |input| {
                            AppMessage::Product(
                                ProductMessage::NameInput(input, false),
                                )
                        })
                        )
                    .push(
                        text_input_column("MSRP", parse_input(&self.product_to_add.msrp), |input| {
                            AppMessage::Product(ProductMessage::MsrpInput(
                                    input,
                                    false
                                    ))
                        })
                        )
                    .push(
                        Column::new()
                          .width(Length::Fill)
                          .align_items(Alignment::Center)
                          .push(
                              Row::new()
                              .spacing(12)
                              .push(self.select_part())
                              .push(self.selected_parts())
                              )
                         )
                    .push(
                        Button::new("Submit")
                        .on_press(AppMessage::Product(ProductMessage::Submit(false)))
                        .style(CustomMainButtonStyle)
                        ),
                        )
                            .into(),
                            )
        } else {
            None
        }
    }

    pub fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_product {
            Some(
                Container::new(
                    Column::new()
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                        .push(
                            Column::new()
                                .max_width(700)
                                .push(
                                    Text::new("Edit Product".to_string())
                                        .size(24)
                                        .horizontal_alignment(Horizontal::Center)
                                        .width(Length::Fill),
                                )
                                .push(
                                    Text::new("Name".to_string())
                                        .horizontal_alignment(Horizontal::Left)
                                        .width(Length::Fill),
                                )
                                .push(
                                    Row::new()
                                        .push(
                                            TextInput::new("Name", &self.product_to_edit.name)
                                                .on_input(|input| {
                                                    AppMessage::Product(
                                                        ProductMessage::NameInput(input, true),
                                                    )
                                                }),
                                        )
                                        .padding([0, 0, 12, 0]),
                                )
                                .push(
                                    Text::new("MSRP".to_string())
                                        .horizontal_alignment(Horizontal::Left)
                                        .width(Length::Fill),
                                )
                                .push(
                                    Row::new()
                                        .push(
                                            TextInput::new(
                                                "MSRP",
                                                &format!("{:.2}", self.product_to_edit.msrp),
                                            )
                                            .on_input(
                                                |input| {
                                                    AppMessage::Product(
                                                        ProductMessage::MsrpInput(input, true),
                                                    )
                                                },
                                            ),
                                        )
                                        .padding([0, 0, 12, 0]),
                                )
                                .push(
                                    Row::new()
                                        .push(
                                            Button::new(
                                                Text::new("Submit".to_string())
                                                    .horizontal_alignment(Horizontal::Center),
                                            )
                                            .on_press(AppMessage::Product(ProductMessage::Submit(true)))
                                            .style(CustomMainButtonStyle)
                                            .width(Length::Fill),
                                        )
                                        .push(Column::new().width(Length::Fill))
                                        .push(
                                            Button::new(
                                                Text::new("Delete".to_string())
                                                    .horizontal_alignment(Horizontal::Center),
                                            )
                                            .on_press(AppMessage::Product(ProductMessage::Delete))
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
                        .push(edit_column("Name", &self.part_to_create.name, |input| {
                            AppMessage::Product(ProductMessage::PartName(input))
                        }))
                        .push(
                            Button::new("Submit")
                                .on_press(AppMessage::Product(ProductMessage::CreatePartSubmit))
                                .style(CustomMainButtonStyle)
                        ),
                ))
                .into(),
            )
        } else {
            None
        }
    }
    

    
    fn view_product(&self) -> Option<Element<AppMessage>> {
        if self.view_product {
            Some(Container::new(
                    Row::new()
                    .push(Column::new()
                          .push(close_button(AppMessage::Product(ProductMessage::CloseView)))
                          .push(Row::new()
                                .push(Text::new("Product")))
                          .push(Row::new()
                                .push(Text::new(&self.product_to_view.name)))
                          .push(Row::new()
                                .push(Text::new("Parts")))
                          .push(Column::new()
                                .extend(self.product_parts_to_view
                                        .iter()
                                        .map(|part|
                                             Row::new()
                                             .push(part_view(&part))
                                             .padding([8,0,8,0])
                                             .into()
                                            )))
                          .padding([0,12,0,0]))
                    ).into())
        } else {
            None
        }
    }
}

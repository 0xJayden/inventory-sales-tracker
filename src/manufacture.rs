use std::env;

use iced::{
    alignment::Horizontal,
    widget::{Button, Column, Container, Row, Scrollable, Text, TextInput},
    Alignment, Element, Length,
};
use sqlx::SqlitePool;

use crate::{
    components::{
        add_button, bold_text, close_button, layout, table_column, table_header, table_row_style,
        table_style, text_input_column, CustomButtonStyle, CustomMainButtonStyle,
    },
    error::Errorr,
    product::Product,
    AppMessage,
};

#[derive(Default, Clone, Debug)]
pub struct Manufacture {
    pub id: i64,
    pub date: String,
}

#[derive(Default, Clone, Debug)]
pub struct ManufactureToAdd {
    pub date: String,
    pub qty: i64,
}

#[derive(Default, Clone, Debug)]
pub struct ProductToSelect {
    pub product_id: i64,
    pub name: String,
    pub qty: i64,
}

#[derive(Default, Clone)]
pub struct ManufactureState {
    pub manufactures: Vec<Manufacture>,
    pub manufacture_to_add: ManufactureToAdd,
    add_manufacture: bool,
    pub manufacture_to_edit: Manufacture,
    pub edit_manufacture: bool,
    pub products: Vec<Product>,
    pub products_to_select: Vec<ProductToSelect>,
    pub products_to_add: Vec<ProductToSelect>,
    query: String,
    pub filtered_products: Vec<ProductToSelect>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ManufactureMessage {
    DateInput(String, bool),
    ShowAddManufacture,
    ShowEditManufacture,
    ProductQtyChanged(String, i64),
    RemoveProduct(i64),
    Submit(bool),
    Delete,
    Query(String),
}

pub fn select_header() -> Container<'static, AppMessage> {
    Container::new(
        Row::new()
            .padding(8)
            .width(Length::Fill)
            .push(bold_text("Name").width(150))
            .push(bold_text("Qty")),
    )
    .style(table_row_style())
}

pub async fn get_manufactures() -> Result<Vec<Manufacture>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let manufactures = sqlx::query_as!(Manufacture, "SELECT * FROM Manufacture")
        .fetch_all(&pool)
        .await?;

    Ok(manufactures)
}

impl ManufactureState {
    pub async fn add_manufacture(
        products_to_add: Vec<ProductToSelect>,
        manufacture_to_add: ManufactureToAdd,
        products: Vec<Product>,
    ) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let date = manufacture_to_add.date;

        let r = sqlx::query!(
            "
            INSERT INTO Manufacture (date)
            VALUES (?)
            ",
            date,
        )
        .execute(&pool)
        .await?;

        let manufacture_id = r.last_insert_rowid();

        for product in &products_to_add {
            sqlx::query!(
                "
                INSERT INTO ManufactureProduct (qty, manufacture_id, product_id)
                VALUES (?,?,?)
                ",
                product.qty,
                manufacture_id,
                product.product_id
            )
            .execute(&pool)
            .await?;

            let productt = products
                .iter()
                .find(|p| p.product_id == product.product_id)
                .unwrap();
            let units = productt.units + product.qty;

            sqlx::query!(
                "
                UPDATE Product
                SET units = ? 
                WHERE product_id = ?
                ",
                units,
                product.product_id
            )
            .execute(&pool)
            .await?;

            let product_parts = sqlx::query!(
                "
                SELECT ProductPart.part_id, ProductPart.qty, Part.units_left, ProductPart.product_id 
                FROM ProductPart 
                JOIN Part ON Part.part_id = ProductPart.part_id
                WHERE product_id = ?
                ",
                product.product_id
                )
                .fetch_all(&pool)
                .await?;

            for product_part in &product_parts {
                let units_left = product_part.units_left - product_part.qty;

                sqlx::query!(
                    "
                    UPDATE Part
                    SET  units_left = ? 
                    WHERE part_id = ?
                    ",
                    units_left,
                    product_part.part_id
                )
                .execute(&pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn edit_manufacture(manufacture: Manufacture) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = manufacture.id;
        let date = manufacture.date;

        sqlx::query!(
            "
            UPDATE Manufacture
            SET date = ?
            WHERE id = ?
            ",
            date,
            id
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    fn select_product(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .spacing(8)
                .push(bold_text("Add Products"))
                .push(
                    TextInput::new("Search", &self.query).on_input(|input| {
                        AppMessage::Manufacture(ManufactureMessage::Query(input))
                    }),
                )
                .push(Container::new(
                    Column::new()
                        .spacing(4)
                        .push(select_header())
                        .push(Scrollable::new(Column::new().spacing(4).extend(
                            self.filtered_products.iter().map(|product| {
                                Container::new(
                                    Row::new()
                                        .width(Length::Fill)
                                        .padding(8)
                                        .spacing(4)
                                        .push(
                                            Column::new()
                                                .push(Row::new().push(table_column(&product.name))),
                                        )
                                        .push(
                                            TextInput::new("Quantity", &product.qty.to_string())
                                                .width(50)
                                                .on_input(|input| {
                                                    AppMessage::Manufacture(
                                                        ManufactureMessage::ProductQtyChanged(
                                                            input,
                                                            product.product_id,
                                                        ),
                                                    )
                                                }),
                                        ),
                                )
                                .style(table_row_style())
                                .into()
                            }),
                        ))),
                )),
        )
        .max_height(300)
    }

    fn selected_products(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
                .spacing(8)
                .width(Length::Fill)
                .push(bold_text("Selected Products"))
                .push(Container::new(
                    Column::new()
                        .spacing(4)
                        .width(Length::Fill)
                        .push(select_header())
                        .push(Scrollable::new(Column::new().spacing(4).extend(
                            self.products_to_add.iter().map(|product| {
                                Container::new(
                                    Row::new()
                                        .width(Length::Fill)
                                        .spacing(4)
                                        .padding(8)
                                        .push(
                                            Column::new()
                                                .width(100)
                                                .align_items(Alignment::Center)
                                                .push(Text::new(&product.name)),
                                        )
                                        .push(
                                            Column::new()
                                                .width(50)
                                                .align_items(Alignment::Center)
                                                .push(Text::new(product.qty.to_string())),
                                        )
                                        .push(close_button(AppMessage::Manufacture(
                                            ManufactureMessage::RemoveProduct(product.product_id),
                                        ))),
                                )
                                .style(table_row_style())
                                .into()
                            }),
                        ))),
                )),
        )
        .max_height(300)
    }

    pub fn update(&mut self, message: ManufactureMessage) {
        match message {
            ManufactureMessage::DateInput(d, is_edit) => {
                if is_edit {
                    self.manufacture_to_edit.date = d;
                } else {
                    self.manufacture_to_add.date = d;
                }
            }
            ManufactureMessage::ShowAddManufacture => {
                if self.add_manufacture {
                    self.add_manufacture = false;
                } else {
                    self.add_manufacture = true;
                }
            }
            ManufactureMessage::ShowEditManufacture => {
                if self.edit_manufacture {
                    self.edit_manufacture = false;
                } else {
                    self.edit_manufacture = true;
                }
            }
            ManufactureMessage::ProductQtyChanged(q, id) => {
                if let Some(i) = self
                    .filtered_products
                    .iter_mut()
                    .find(|item| item.product_id == id)
                {
                    i.qty = q.parse::<i64>().unwrap_or(0);
                    match self
                        .products_to_add
                        .iter_mut()
                        .find(|item| item.product_id == id)
                    {
                        Some(p) => {
                            p.qty = i.qty;
                            if p.qty == 0 {
                                let f_products =
                                    self.products_to_add.iter().filter_map(|product| {
                                        match product.product_id != id {
                                            true => Some(product.to_owned()),
                                            false => None,
                                        }
                                    });

                                self.products_to_add = f_products.collect();
                            }
                        }
                        None => self.products_to_add.push(i.clone()),
                    }
                }
            }
            ManufactureMessage::RemoveProduct(id) => {
                let f_products = self.products_to_add.iter().filter_map(|product| {
                    if product.product_id != id {
                        Some(product.to_owned())
                    } else {
                        None
                    }
                });

                self.products_to_add = f_products.collect();
            }
            ManufactureMessage::Submit(is_edit) => match is_edit {
                true => {
                    self.edit_manufacture = false;
                }
                false => {
                    self.add_manufacture = false;
                }
            },
            ManufactureMessage::Delete => {
                println!("Deleting...")
            }
            ManufactureMessage::Query(q) => {
                if q.len() > 0 {
                    self.filtered_products = self
                        .products_to_select
                        .iter()
                        .filter_map(|product| {
                            if product.name.contains(&q) {
                                Some(product.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect();
                } else {
                    self.filtered_products = self.products_to_select.clone()
                }
                self.query = q;
            }
        }
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
                .width(Length::Fill)
                .padding([12, 0, 0, 12])
                .align_items(Alignment::Center)
                .push(Text::new("Manufactures".to_string()).size(24))
                .push(
                    Row::new()
                        .push(add_button(
                            "Add Manufacture",
                            AppMessage::Manufacture(ManufactureMessage::ShowAddManufacture),
                        ))
                        .padding(12),
                )
                .push_maybe(self.create_view())
                .push_maybe(self.edit_view())
                .push(
                    Container::new(table_header(&["Date"]).push(
                        Scrollable::new(Column::new().padding([0, 8, 0, 0]).extend(
                            self.manufactures.iter().map(|manufacture| {
                                Button::new(
                                    Container::new(
                                        Row::new()
                                            .padding(10)
                                            .push(table_column(&manufacture.date)),
                                    )
                                    .style(table_row_style()),
                                )
                                .style(CustomButtonStyle)
                                .on_press(AppMessage::EditManufacture(manufacture.clone()))
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

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_manufacture {
            Some(
                Column::new()
                    .max_width(1000)
                    .push(
                        Container::new(
                            Column::new()
                                .spacing(12)
                                .push(
                                    Text::new("Add Manufacture".to_string())
                                        .size(24)
                                        .horizontal_alignment(Horizontal::Center)
                                        .width(Length::Fill),
                                )
                                .push(text_input_column(
                                    "Date",
                                    &self.manufacture_to_add.date,
                                    |input| {
                                        AppMessage::Manufacture(ManufactureMessage::DateInput(
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
                                                .push(self.select_product())
                                                .push(self.selected_products()),
                                        ),
                                )
                                .push(
                                    Button::new("Submit")
                                        .on_press(AppMessage::Manufacture(
                                            ManufactureMessage::Submit(false),
                                        ))
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

    pub fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_manufacture {
            Some(
                Container::new(
                    Column::new()
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                        .push(
                            Column::new()
                                .max_width(700)
                                .push(
                                    Text::new("Edit Manufacture".to_string())
                                        .size(24)
                                        .horizontal_alignment(Horizontal::Center)
                                        .width(Length::Fill),
                                )
                                .push(text_input_column(
                                    "Date",
                                    &self.manufacture_to_edit.date,
                                    |input| {
                                        AppMessage::Manufacture(ManufactureMessage::DateInput(
                                            input, true,
                                        ))
                                    },
                                    Some(AppMessage::Manufacture(ManufactureMessage::Submit(true))),
                                ))
                                .push(
                                    Row::new()
                                        .push(
                                            Button::new(
                                                Text::new("Submit".to_string())
                                                    .horizontal_alignment(Horizontal::Center),
                                            )
                                            .on_press(AppMessage::Manufacture(
                                                ManufactureMessage::Submit(true),
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
                                            .on_press(AppMessage::Manufacture(
                                                ManufactureMessage::Delete,
                                            ))
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
}

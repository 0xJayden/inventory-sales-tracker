use crate::{components::layout, error::Errorr, parts::Part, product::Product, AppMessage};
use sqlx::SqlitePool;
use std::env;

use iced::{
    border::Radius,
    widget::{container, Column, Container, Row, Text},
    Alignment, Background, Border, Color, Element, Length, Vector,
};

#[derive(Default, Clone)]
pub struct HomeState {
    pub sales: Vec<SaleH>,
    pub products: Vec<Product>,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HomeMessage {}

fn alert_view(len: usize, text: String) -> Option<Element<'static, AppMessage>> {
    if len > 0 {
        Some(
            card(Container::new(Column::new().push(Row::new().push(Text::new(text)))).into())
                .into(),
        )
    } else {
        None
    }
}

fn card(view: Element<AppMessage>) -> Container<AppMessage> {
    Container::new(Column::new().width(Length::Fill).padding(8).push(view)).style(
        container::Appearance {
            text_color: Some(Color::BLACK),
            background: Some(Background::Color(Color::new(1.0, 1.0, 0.5, 1.0))),
            border: Border {
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                width: 1.0,
                radius: Radius::from(5.0),
            },
            shadow: iced::Shadow {
                color: Color::new(0.5, 0.5, 0.5, 1.0),
                offset: Vector { x: 1.0, y: 1.0 },
                blur_radius: 10.0,
            },
        },
    )
}

#[derive(Clone, Debug)]
pub struct SPS {
    pub sales: Vec<SaleH>,
    pub products: Vec<Product>,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug)]
pub struct SaleH {
    pub sale_id: i64,
    pub discount: Option<f64>,
    pub total: f64,
    pub cost: f64,
    pub net: f64,
    pub date: String,
    pub client_id: i64,
    pub rep_cut: Option<f64>,
    pub status: String,
    pub shipping: f64,
    pub rep_id: Option<i64>,
    pub note: Option<String>,
}

pub async fn get_home() -> Result<SPS, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let sales = sqlx::query_as!(
        SaleH,
        "
        SELECT * FROM Sale
        WHERE status = ?
        ",
        "DRAFT"
    )
    .fetch_all(&pool)
    .await?;

    let products = sqlx::query_as!(
        Product,
        "
        SELECT * FROM Product
        WHERE units <= 25
        "
    )
    .fetch_all(&pool)
    .await?;

    let parts = sqlx::query_as!(
        Part,
        "
        SELECT * FROM Part
        WHERE units_left <= 25
        "
    )
    .fetch_all(&pool)
    .await?;

    let r = SPS {
        sales,
        products,
        parts,
    };

    Ok(r)
}

impl HomeState {
    pub fn update(&mut self, message: HomeMessage) {
        match message {}
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
                .padding([12, 0, 0, 12])
                .align_items(Alignment::Center)
                .push(
                    Column::new()
                        .padding([0, 0, 12, 0])
                        .push(Text::new("Home".to_string()).size(24)),
                )
                .push(
                    Column::new()
                        .spacing(8)
                        .push_maybe(alert_view(
                            self.sales.len(),
                            format!("{} Sales Need to be Completed", self.sales.len()),
                        ))
                        .push_maybe(alert_view(
                            self.products.len(),
                            format!(
                                "{} Products are Low and Need to be Made",
                                self.products.len()
                            ),
                        ))
                        .push_maybe(alert_view(
                            self.parts.len(),
                            format!(
                                "{} Parts are Low and Need to be Purchased",
                                self.parts.len()
                            ),
                        )),
                )
                .into(),
        )
        .into()
    }
}

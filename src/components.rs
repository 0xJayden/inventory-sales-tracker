use std::env;

use iced::{
    alignment::Horizontal,
    font::Weight,
    widget::{button, container, svg, Button, Column, Container, Row, Text, TextInput},
    Alignment, Background, Border, Color, Element, Font, Length, Vector,
};

use crate::AppMessage;

const MAIN_COLOR: Color = Color {
    r: 0.5,
    g: 0.2,
    b: 0.5,
    a: 1.0,
};

const BG_COLOR: Color = Color {
    r: 0.9,
    g: 0.9,
    b: 0.9,
    a: 1.0,
};

const BORDER_COLOR: Color = Color {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 1.0,
};

const SHADOW_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 0.9,
};

const TEXT_COLOR: Color = Color {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};

pub fn bold_text(s: &str) -> Text {
    Text::new(s.to_string()).font(Font {
        weight: Weight::Bold,
        ..Default::default()
    })
}

pub fn text_input_column<'a, F>(
    label: &'static str,
    value: &'a str,
    on_input: F,
    on_submit: Option<AppMessage>,
) -> Column<'static, AppMessage>
where
    F: 'static + Fn(String) -> AppMessage,
{
    Column::new()
        .spacing(4)
        .push(bold_text(label))
        .push(if let Some(msg) = on_submit {
            TextInput::new(label, value)
                .on_input(on_input)
                .on_submit(msg)
        } else {
            TextInput::new(label, value).on_input(on_input)
        })
}

pub struct CustomButtonStyle;

impl button::StyleSheet for CustomButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}

impl Into<iced::theme::Button> for CustomButtonStyle {
    fn into(self) -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(CustomButtonStyle))
    }
}

pub struct CustomMainButtonStyle;

impl button::StyleSheet for CustomMainButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(MAIN_COLOR)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

impl Into<iced::theme::Button> for CustomMainButtonStyle {
    fn into(self) -> iced::theme::Button {
        iced::theme::Button::Custom(Box::new(CustomMainButtonStyle))
    }
}

struct CustomSvgStyle;

impl svg::StyleSheet for CustomSvgStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance {
            color: Some(TEXT_COLOR),
        }
    }
}

impl Into<iced::theme::Svg> for CustomSvgStyle {
    fn into(self) -> iced::theme::Svg {
        iced::theme::Svg::Custom(Box::new(CustomSvgStyle))
    }
}

struct CustomTextStyle;

impl Into<iced::theme::Text> for CustomTextStyle {
    fn into(self) -> iced::theme::Text {
        iced::theme::Text::Color(TEXT_COLOR)
    }
}

pub fn add_button(text: &str, msg: AppMessage) -> Button<AppMessage> {
    Button::new(text).on_press(msg).style(CustomMainButtonStyle)
}

fn navbar_button(handle: svg::Handle, text: &str, msg: AppMessage) -> Button<AppMessage> {
    Button::new(
        Row::new()
            .spacing(8)
            .push(svg(handle).width(24).height(24).style(CustomSvgStyle))
            .push(Text::new(text).style(CustomTextStyle)),
    )
    .on_press(msg)
    .style(CustomButtonStyle)
}

fn svg_handle(path: &str) -> svg::Handle {
    let exe = env::current_exe().unwrap();

    // MacOS
    // let contents = exe.parent().unwrap().parent().unwrap().to_str().unwrap();
    // let h = svg::Handle::from_path(format!("{}/Resources/{}.svg", contents, path));

    let contents = exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    let h = svg::Handle::from_path(format!("{}/assets/{}.svg", contents, path));

    h
}

fn navbar() -> Container<'static, AppMessage> {
    let home = svg_handle("home");
    let purchase = svg_handle("purchase");
    let manufacture = svg_handle("manufacture");
    let part = svg_handle("part");
    let product = svg_handle("product");
    let sale = svg_handle("sale");
    let client = svg_handle("client");
    let rep = svg_handle("rep");

    Container::new(
        Column::new()
            .spacing(20)
            .padding(20)
            .push(navbar_button(home, "Home", AppMessage::GoToHome))
            .push(navbar_button(
                purchase,
                "Purchases",
                AppMessage::GoToPurchases,
            ))
            .push(navbar_button(
                manufacture,
                "Manufactures",
                AppMessage::GoToManufactures,
            ))
            .push(navbar_button(part, "Parts", AppMessage::GoToParts))
            .push(navbar_button(product, "Products", AppMessage::GoToProducts))
            .push(navbar_button(sale, "Sales", AppMessage::GoToSales))
            .push(navbar_button(client, "Clients", AppMessage::GoToClients))
            .push(navbar_button(rep, "Reps", AppMessage::GoToReps)),
    )
    .style(container::Appearance {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow {
            color: SHADOW_COLOR,
            offset: Vector { x: 1.0, y: 1.0 },
            blur_radius: 10.0,
        },
        ..Default::default()
    })
    .height(Length::Fill)
    .align_x(Horizontal::Center)
}

pub fn layout(content: Element<AppMessage>) -> Container<AppMessage> {
    Container::new(
        Row::new()
            .width(Length::Fill)
            .push(navbar())
            .push(content)
            .height(Length::Fill)
            .padding(12),
    )
    .style(container::Appearance {
        background: Some(Background::Color(BG_COLOR)),
        ..Default::default()
    })
    .into()
}

pub fn card_style() -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn table_style() -> container::Appearance {
    container::Appearance {
        border: Border {
            color: BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow {
            color: SHADOW_COLOR,
            offset: Vector { x: 1.0, y: 1.0 },
            blur_radius: 20.0,
        },
        ..Default::default()
    }
}

pub fn table_row_style() -> container::Appearance {
    container::Appearance {
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        background: Some(Background::Color(Color::WHITE)),
        ..Default::default()
    }
}

pub fn table_row_qty_style(qty: i64) -> container::Appearance {
    container::Appearance {
        text_color: Some(Color::BLACK),
        background: if qty < 10 {
            Some(Background::Color(Color::new(1.0, 0.0, 0.0, 1.0)))
        } else if qty > 10 && qty < 25 {
            Some(Background::Color(Color::new(1.0, 1.0, 0.0, 1.0)))
        } else {
            Some(Background::Color(Color::WHITE))
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn table_column(str: &str) -> Column<'static, AppMessage> {
    Column::new()
        .push(Text::new(str.to_string()))
        .width(150)
        .align_items(Alignment::Center)
}

fn table_label(str: &str) -> Container<'static, AppMessage> {
    Container::new(
        Column::new()
            .width(150)
            .align_items(Alignment::Center)
            .push(Text::new(str.to_string()).size(20)),
    )
}

pub fn table_header(labels: &[&str]) -> Column<'static, AppMessage> {
    Column::new().push(
        Row::new()
            .padding(8)
            .extend(labels.iter().map(|label| table_label(label).into())),
    )
}

pub fn close_button(msg: AppMessage) -> Button<'static, AppMessage> {
    let x = svg_handle("x");

    Button::new(svg(x).width(24).height(24))
        .on_press(msg)
        .style(iced::theme::Button::Destructive)
}

pub fn close_edit_row(close: AppMessage, edit: AppMessage) -> Row<'static, AppMessage> {
    Row::new()
        .width(Length::Fill)
        .push(
            Column::new()
            .width(Length::Fill)
            .push(
                close_button(close)
                )
            )
        .push(
            Column::new()
            .width(Length::Fill)
            .align_items(Alignment::End)
            .push(
                Button::new("Edit")
                .on_press(edit)
                )
            )
}

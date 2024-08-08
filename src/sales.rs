use std::env;
use clipboard::{ClipboardContext, ClipboardProvider};

use iced::{
    alignment::Horizontal,
    widget::{scrollable::{Direction, Properties}, Button, Column, Container, Row, Scrollable, Text, TextInput},
    Alignment, Element, Length, 
};

use sqlx::SqlitePool;

use crate::{
    clients::{get_client, get_clients, Client}, components::{add_button, bold_text, card_style, close_button, edit_column, layout, table_column, table_header, table_row_style, table_style, text_input_column, CustomButtonStyle, CustomMainButtonStyle}, manufacture::select_header, product::{get_products, Product}, rep::{get_reps, Rep}, AppMessage
};

use crate::error::Errorr;

#[derive(Default, Clone, Debug)]
pub struct SaleProduct {
    pub name: String,
    pub qty: i64,
    pub units: i64,
    pub msrp: f64,
    pub cost: f64,
    pub cost_at_sale: f64,
    pub msrp_at_sale: f64
}

#[derive(Clone, Default, Debug)]
pub struct SaleProductToAdd {
    pub product_id: i64,
    pub name: String,
    pub qty: i64,
    pub msrp: f64,
    pub cost: f64,
    pub units: i64
}

#[derive(Debug, Clone, Default)]
pub struct Sale {
    pub sale_id: i64, 
    pub discount: Option<f64>,
    pub total: f64,
    pub cost: f64,
    pub net: f64,
    pub date: String,
    pub client_id: i64,
    pub client_name: String,
    pub note: Option<String>,
    pub rep_id: Option<i64>,
    pub rep_name: String,
    pub rep_percentage: u8,
    pub rep_cut: Option<f64>,
    pub status: String,
    pub shipping: f64
}

#[derive(Default, Clone)]
pub struct SalesState {
    pub sales: Vec<Sale>,
    pub add_sales: Sale,
    pub sales_products: Vec<SaleProduct>,
    pub clients: Vec<Client>,
    pub products_to_add: Vec<SaleProductToAdd>,
    pub edit_sale: bool,
    pub sale_to_edit: Sale,
    pub sale_products_to_view: Vec<SaleProduct>,
    create_client: bool,
    pub client_to_create: Client,
    pub client_to_view: Client,
    pub sale_to_view: Sale,
    pub view_sale: bool,
    query: String,
    pub filtered_products: Vec<SaleProductToAdd>,
    pub products_to_select: Vec<SaleProductToAdd>,
    add_sale: bool,
    pub reps: Vec<Rep>,
    create_rep: bool,
    pub rep_to_create: Rep,
    pub filtered_clients: Vec<Client>,
    client_query: String,
    pub filtered_reps: Vec<Rep>,
    rep_query: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaleMessage {
    ProductQtyChanged(String, i64, f64, f64),
    AddClient(i64, String),
    CreateClient,
    CreateClientSubmit,
    ClientName(String),
    ClientAddress(String),
    ClientEmail(String),
    ViewClient(i64),
    AddRep(i64, String, u8),
    CreateRep,
    CreateRepSubmit,
    RepName(String),
    RepPercentage(String),
    DiscountInput(String, bool),
    DateInput(String, bool),
    NoteInput(String, bool),
    EditClient(i64),
    ShowAddProducts,
    Submit(bool),
    Delete,
    RemoveProduct(i64),
    Query(String),
    ClientQuery(String),
    RepQuery(String),
    CopyClientInfo,
    Fulfill,
    CloseSale
}

pub async fn get_sales() -> Result<Vec<Sale>, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let sales = sqlx::query_as!(Sale,
                                "SELECT Sale.sale_id, discount, total, Sale.cost, Sale.client_id, net, date, note, rep_id, shipping, status, rep_cut,
                                Client.name as client_name,
                                Rep.name as rep_name, Rep.percentage as `rep_percentage: u8`
                                FROM Sale
                                JOIN Client ON Sale.client_id = Client.client_id
                                JOIN Rep ON Sale.rep_id = Rep.id
                                "
                               )
        .fetch_all(&pool)
        .await?;

    Ok(sales)
}

#[derive(Clone, Debug)]
pub struct SC {
    pub sale_products: Vec<SaleProduct>,
    pub client: Client
}

pub async fn get_sale_products_and_client(sale_id: i64, client_id: i64) -> Result<SC, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let sale_products = sqlx::query_as!(SaleProduct,
                               "
                               SELECT SaleProduct.cost_at_sale, SaleProduct.msrp_at_sale, SaleProduct.qty,
                               Product.name, Product.units, Product.cost, Product.msrp
                               FROM SaleProduct
                               JOIN Product ON SaleProduct.product_id = Product.product_id
                               WHERE SaleProduct.sale_id = ?
                               ",
                               sale_id)
        .fetch_all(&pool)
        .await?;

    let client = get_client(client_id).await?;

    let r = SC {
        sale_products,
        client
    };

    Ok(r)
}

#[derive(Clone, Debug)]
pub struct PCR {
    pub products: Vec<Product>,
    pub clients: Vec<Client>,
    pub reps: Vec<Rep>
}

pub async fn get_products_and_clients() -> Result<PCR, Errorr> {
    let products = get_products().await?;
    
    let clients = get_clients().await?;

    let reps = get_reps().await?;

    let r = PCR {
        products,
        clients,
        reps
    };

    Ok(r)
}

pub async fn add_client_set(client: Client) -> Result<i64, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let name = client.name;
    let address = client.address;
    let email = client.email;

    let c = sqlx::query!(
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

    Ok(c.last_insert_rowid())
}

#[derive(Clone, Debug)]
pub struct R {
    pub id: i64,
    pub name: String
}

pub async fn add_rep_set(rep: Rep) -> Result<R, Errorr> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    let name = rep.name;
    let percentage = rep.percentage;

    let c = sqlx::query!(
        "
        INSERT INTO Rep (name, percentage)
        VALUES (?,?)
        ",
        name,
        percentage
        )
        .execute(&pool)
        .await?;

    let r = R {
        id: c.last_insert_rowid(),
        name
    };

    Ok(r)
}

fn item_view_row(label: &str, value: String) -> Row<'static, AppMessage> {
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

fn item_view(item: &SaleProduct) -> Container<'static, AppMessage> {
    let total = item.msrp * item.qty as f64;
    let net = total - item.cost * item.qty as f64;
    
    Container::new(
        Column::new()
        .push(Text::new(item.name.to_string())
              .size(20))
        .push(item_view_row("Quantity: ", item.qty.to_string()))
        .push(item_view_row("Cost: ", format!("${:.2}", item.cost)))
        .push(item_view_row("MSRP: ", format!("${:.2}", item.msrp)))
        .push(item_view_row("Net: ", format!("${:.2}", net)))
        .push(item_view_row("Total: ", format!("${:.2}", total))))
        .padding(8)
        .style(card_style())
}

fn client_view_row(value: String) -> Row<'static, AppMessage> {
    Row::new()
        .padding(4)
        .push(Text::new(value))
}

fn client_view(client: &Client) -> Container<'static, AppMessage> {
    Container::new(
        Column::new()
            .push(Text::new("Client"))
            .push(client_view_row(client.name.clone()))
            .push(client_view_row(client.address.clone()))
            .push(client_view_row(client.email.clone().unwrap_or("".to_string())))
            .push(Button::new("Copy Client Info")
                  .on_press(AppMessage::Sale(SaleMessage::CopyClientInfo))
                  .style(CustomMainButtonStyle))
        )
}

impl SalesState { 
    pub async fn edit_sale(sale: Sale) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = sale.sale_id;
        let discount  = sale.discount.unwrap_or(0.00);
        let date = sale.date;
        let client = sale.client_id;
        let note = sale.note;

        sqlx::query!(
            "
            UPDATE Sale
            SET discount = ?, date = ?, client_id = ?, note = ?
            WHERE sale_id = ?
            ",
            discount,
            date,
            client,
            note,
            id
            )
            .execute(&pool)
            .await?;

        Ok(())
    }

    pub async fn fulfill_sale(id: i64) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        sqlx::query!(
            "
            UPDATE Sale
            SET status = ?
            WHERE sale_id = ?
            ",
            "COMPLETED",
            id
            )
            .execute(&pool)
            .await?;

        Ok(())
    }

    pub async fn delete_sale(sale: Sale) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

        let id = sale.sale_id;

        sqlx::query!(
            "
            DELETE FROM Sale
            WHERE sale_id = ?
            ",
            id
            )
            .execute(&pool)
            .await?;

        Ok(())
    }

    pub async fn add_sales(i: Vec<SaleProductToAdd>, j: Vec<SaleProductToAdd>, sales: Sale) -> Result<(), Errorr> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
        let discount = sales.discount;
        let total = sales.total;
        let cost = sales.cost;
        let net = sales.net;
        let date = sales.date;
        let client = sales.client_id;
        let note = sales.note;
        let rep = sales.rep_id;
        let rep_cut = sales.rep_cut;
        let shipping = sales.shipping;

        let sale = sqlx::query!(
            "
            INSERT INTO Sale ( discount, total, cost, net, date, client_id, note, rep_id, rep_cut, shipping )
            VALUES (?,?,?,?,?,?,?,?,?,?)
            ",
            discount,
            total,
            cost,
            net,
            date,
            client,
            note,
            rep,
            rep_cut,
            shipping
            )
            .execute(&pool)
            .await?;

        let sale_id = sale.last_insert_rowid();

        for item in &j {
            sqlx::query!(
                "
                INSERT INTO SaleProduct ( sale_id, qty, product_id, cost_at_sale, msrp_at_sale )
                VALUES (?,?,?,?,?)
                ",
                sale_id,
                item.qty,
                item.product_id,
                item.cost,
                item.msrp
                )
                .execute(&pool)
                .await?;

            let units = i.iter().find(|i| i.product_id == item.product_id).unwrap().units - item.qty;

            sqlx::query!(
                    "
                    UPDATE Product
                    SET units = ?
                    WHERE product_id = ?
                    ",
                    units,
                    item.product_id
                )
                .execute(&pool)
                .await?;
        };

        Ok(())
    }
    
    pub fn update(&mut self, message: SaleMessage) {
       match message {
            SaleMessage::ProductQtyChanged(qty, id, cost, msrp) => {
                if let Some(i) = self.filtered_products.iter_mut().find(|item| item.product_id == id) {
                    i.qty = qty.parse::<i64>().unwrap_or(0);
                    i.cost = cost;
                    i.msrp = msrp;

                    match self.products_to_add.iter_mut().find(|item| item.product_id == id) {
                        Some(p) => {
                            p.qty = i.qty;
                            if p.qty == 0 {
                                let f_products = self.products_to_add.iter().filter_map(|product| match product.product_id != id {
                                    true => Some(product.to_owned()),
                                    false => None
                                });
                
                                self.products_to_add = f_products.collect();
                            }
                        }
                        None => {
                            self.products_to_add.push(i.clone())
                        }
                    }
                }
            }
            SaleMessage::DiscountInput(d, is_edit) => {
                if is_edit {
                    self.sale_to_edit.discount = Some(d.parse::<f64>().unwrap_or(0.00))
                } else {
                    let discount = d.parse::<f64>().unwrap_or(0.00);

                    if discount == 0.00 {
                        self.add_sales.discount = None;
                    } else {
                        self.add_sales.discount = Some(discount);
                    }
                }
            }
            SaleMessage::DateInput(d, is_edit) => {
                if is_edit {
                    self.sale_to_edit.date = d;
                } else {
                    self.add_sales.date = d;
                }
            }
            SaleMessage::AddClient(cid, cname) => {
                self.add_sales.client_id = cid;
                self.add_sales.client_name = cname;
            }
            SaleMessage::CreateClient => {
                if self.create_client {
                    self.create_client = false;
                } else {
                    self.create_client = true;
                }
            }
            SaleMessage::CreateClientSubmit => {
                self.add_sales.client_id = self.client_to_create.client_id;
                self.create_client = false;
            }
            SaleMessage::ClientName(s) => {
                self.client_to_create.name = s;
            }
            SaleMessage::ClientAddress(a) => {
                self.client_to_create.address = a;
            }
            SaleMessage::ClientEmail(e) => {
                self.client_to_create.email = Some(e);
            }
            SaleMessage::ViewClient(i) => {
                self.client_to_view.client_id = i;
            }
            SaleMessage::AddRep(cid, cname, p) => {
                self.add_sales.rep_id = Some(cid);
                self.add_sales.rep_name = cname;
                self.add_sales.rep_percentage = p; 
            }
            SaleMessage::CreateRep => {
                if self.create_rep {
                    self.create_rep = false;
                } else {
                    self.create_rep = true;
                }
            }
            SaleMessage::CreateRepSubmit => {
                self.add_sales.rep_id = Some(self.rep_to_create.id);
                self.create_rep = false;
            }
            SaleMessage::RepName(s) => {
                self.rep_to_create.name = s;
            }
            SaleMessage::RepPercentage(p) => {
                self.rep_to_create.percentage = p.parse::<u8>().unwrap_or(0); 
            }
            SaleMessage::NoteInput(n, is_edit) => {
                if is_edit {
                    if n.chars().count() == 0 {
                        self.add_sales.note = None;
                    } else {
                        self.add_sales.note = Some(n);
                    }
                } else {
                    self.sale_to_edit.note = Some(n);
                }
            }
            SaleMessage::ShowAddProducts => {
                if self.add_sale {
                    self.add_sale = false;
                } else {
                    self.view_sale = false;
                    self.add_sale = true;
                }
            }
            SaleMessage::EditClient(c) => {
                self.sale_to_edit.client_id = c;
            }
            SaleMessage::Delete => {
                self.edit_sale = false;
            }
            SaleMessage::Submit(is_edit) => {
                if is_edit {
                    self.edit_sale = false;
                } else {
                    self.products_to_add.iter_mut().for_each(|item| {
                        self.add_sales.cost += item.cost;
                        let total = item.msrp * item.qty as f64;
                        self.add_sales.total += total;
                        self.add_sales.net += total - (item.cost * item.qty as f64);
                    });

                    if self.add_sales.total >= 500.00 {
                        self.add_sales.shipping = 0.00;
                    } else {
                        self.add_sales.shipping = 15.00;
                    }

                    if let Some(_) = self.add_sales.rep_id {
                        let rep_cut = self.add_sales.total * (self.add_sales.rep_percentage as f64 / 100.00);
                        let new_net = self.add_sales.net - rep_cut;
                        self.add_sales.net = new_net;
                        self.add_sales.rep_cut = Some(rep_cut);
                    }

                    self.add_sales.total += self.add_sales.shipping;
                    self.add_sales.cost += 9.00; // 9.00 cost to ship
                    self.add_sales.net += self.add_sales.shipping - 9.00; // 9.00 cost to ship

                    self.add_sale = false;
                }
            }
            SaleMessage::Query(q) => {
                if q.len() > 0 {
                    self.filtered_products = self.products_to_select.iter().filter_map(|product| if product.name.contains(&q) {
                        Some(product.to_owned())
                    } else {
                        None
                    }
                    ).collect();
                } else {
                    self.filtered_products = self.products_to_select.clone()
                }
                self.query = q;
            }
            SaleMessage::ClientQuery(q) => {
                if q.len() > 0 {
                    self.filtered_clients = self.clients.iter().filter_map(|client| if client.name.contains(&q) {
                        Some(client.to_owned())
                    } else {
                        None
                    }
                    ).collect();
                } else {
                    self.filtered_clients = self.clients.clone()
                }
                self.client_query = q;
            }
            SaleMessage::RepQuery(q) => {
                if q.len() > 0 {
                    self.filtered_reps = self.reps.iter().filter_map(|rep| if rep.name.contains(&q) {
                        Some(rep.to_owned())
                    } else {
                        None
                    }
                    ).collect();
                } else {
                    self.filtered_reps = self.reps.clone()
                }
                self.rep_query = q;
            }
            SaleMessage::RemoveProduct(id) => {
                let f_products =
                    self.products_to_add
                        .iter()
                        .filter_map(|product| 
                                    if product.product_id != id {
                                        Some(product.to_owned())
                                    } else {
                                        None
                                    }
                        );

                self.products_to_add = f_products.collect();
            }
            SaleMessage::CopyClientInfo => {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let contents = format!("{} {}", self.client_to_view.name, self.client_to_view.address);
                ctx.set_contents(contents).unwrap();
            }
            SaleMessage::Fulfill => {
                println!("fulfilling")
            }
            SaleMessage::CloseSale => {
                self.view_sale = false;
            }
        }
    }
    
    fn select_rep(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .push(
                Row::new()
                .push(bold_text("Select Rep"))
                .push(
                    Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .push(
                    Button::new("New Rep")
                      .on_press(AppMessage::Sale(SaleMessage::CreateRep))
                      .style(CustomMainButtonStyle)
                      )
                    )
                )
                .push(
                    TextInput::new("Search", &self.rep_query)
                    .on_input(|input| AppMessage::Sale(SaleMessage::RepQuery(input)))
                    )
            .push(
                Container::new(
                    Column::new()
                    .width(Length::Fill)
                    .push_maybe(self.create_rep_view())
                    .push(
                        Scrollable::new(
                            Column::new()
                            .padding(12)
                            .extend(self.filtered_reps.iter().map(
                                    |rep|
                                        Column::new()
                                        .push(
                                            Button::new(table_column(&rep.name))
                                            .width(Length::Fill)
                                            .style(CustomButtonStyle)
                                          .on_press(AppMessage::Sale(SaleMessage::AddRep(rep.id, rep.name.clone(), rep.percentage))))
                                    .into()
                                    )))
                    ))
                    .max_height(200)
                    .style(card_style())))
    }
    
    fn select_client(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .push(
                Row::new()
                .push(bold_text("Select Client"))
                .push(
                    Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .push(
                    Button::new("New Client")
                      .on_press(AppMessage::Sale(SaleMessage::CreateClient))
                      .style(CustomMainButtonStyle)
                      )
                    )
                )
                .push(
                    TextInput::new("Search", &self.client_query)
                    .on_input(|input| AppMessage::Sale(SaleMessage::ClientQuery(input)))
                    )
            .push(
                Container::new(
                    Column::new()
                    .width(Length::Fill)
                    .push_maybe(self.create_client_view())
                    .push(
                        Scrollable::new(
                            Column::new()
                            .padding(12)
                            .extend(self.filtered_clients.iter().map(
                                    |client|
                                        Column::new()
                                        .push(
                                            Button::new(table_column(&client.name))
                                            .width(Length::Fill)
                                            .style(CustomButtonStyle)
                                          .on_press(AppMessage::Sale(SaleMessage::AddClient(client.client_id, client.name.clone()))))
                                    .into()
                                    )))
                    ))
                    .max_height(200)
                    .style(card_style())))
    }
    
    fn selected_rep(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .push(bold_text("Selected Rep"))
            .push(
                Container::new(
                    Column::new()
                    .padding(12)
                    .width(Length::Fill)
                    .push(Column::new()
                          .push(Text::new(self.add_sales.rep_name.clone())))
                    )
                .style(card_style())))
    }

    fn selected_client(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .push(bold_text("Selected Client"))
            .push(
                Container::new(
                    Column::new()
                    .padding(12)
                    .width(Length::Fill)
                    .push(Column::new()
                          .push(Text::new(self.add_sales.client_name.clone())))
                    )
                .style(card_style())))
    }

    fn select_product(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .push(
                Row::new()
                .width(Length::Fill)
                .push(bold_text("Add Products"))
                )
            .push(
                TextInput::new("Search", &self.query)
                .on_input(|input| AppMessage::Sale(SaleMessage::Query(input)))
                )
            .push(
                Container::new(
                    Column::new()
                    .spacing(4)
                    .width(Length::Fill)
                    .push(select_header())
                    .push(
                        Scrollable::new(
                            Column::new()
                            .spacing(4)
                            .extend(
                                self.filtered_products
                                .iter()
                                .map(|product| {
                                    Container::new(
                                        Row::new()
                                        .width(Length::Fill)
                                        .spacing(4)
                                        .padding(8)
                                        .push(
                                            Column::new()
                                            .push(Row::new()
                                                  .push(table_column(&product.name))),
                                                  )
                                        .push(
                                            TextInput::new("Quantity", &product.qty.to_string())
                                            .width(50)
                                            .on_input(|input| {
                                                AppMessage::Sale(SaleMessage::ProductQtyChanged(
                                                        input,
                                                        product.product_id,
                                                        product.cost,
                                                        product.msrp
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

    fn selected_products(&self) -> Container<'_, AppMessage> {
        Container::new(
            Column::new()
            .spacing(8)
            .width(Length::Fill)
            .push(bold_text("Selected Products"))
                  .push(
                      Container::new(
                          Column::new()
                          .width(Length::Fill)
                          .spacing(4)
                          .push(select_header())
                          .push(
                              Scrollable::new(
                                  Column::new()
                                  .extend(
                                      self.products_to_add
                                      .iter()
                                      .map(|product| {
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
                                                  .push(
                                                      Text::new(product.qty.to_string())
                                                      )
                                                  )
                                              .push(close_button(AppMessage::Sale(SaleMessage::RemoveProduct(product.product_id)))
                                                   ))
                                                   .style(table_row_style())
                                                   .into()
                                      }),
                                      ))))
                                          ),
                                          )
                                              .max_height(300)
    }

    pub fn view(&self) -> Element<AppMessage> {
        layout(
            Column::new()
            .width(Length::Fill)
            .padding([12, 0, 0, 12])
            .align_items(Alignment::Center)
            .push(Text::new("Sales".to_string())
                  .size(24))
            .push(
                Row::new()
                  .push(
                    add_button("Add Sale", AppMessage::Sale(SaleMessage::ShowAddProducts))
                    )
                  .padding(12))
            .push_maybe(self.view_sale())
            .push_maybe(self.create_view())
            .push_maybe(self.edit_view())
            .push(
                Row::new()
                .padding(10)
                .push(
                    Container::new(
                        Scrollable::new(
                            table_header(&["Status", "Date", "Discount", "Shipping", "Total", "Cost", "Rep Cut", "Net", "Client", "Rep", "Note"])
                            .push(
                                Scrollable::new(
                                    Column::new()
                                    .padding([0,8,0,0])
                                    .extend(self.sales.iter().map(
                                            |item| 
                                            Button::new(
                                                Container::new(
                                                    Column::new()
                                                    .push(
                                                        Row::new()
                                                        .padding(10)
                                                        .push(table_column(&item.status))
                                                        .push(table_column(&item.date))
                                                        .push(table_column(&format!("${:.2}", &item.discount.unwrap_or(0.00))))
                                                        .push(table_column(&format!("${:.2}", &item.shipping)))
                                                        .push(table_column(&format!("${:.2}", &item.total)))
                                                        .push(table_column(&format!("${:.2}", &item.cost)))
                                                        .push(table_column(&format!("${:.2}", &item.rep_cut.unwrap_or(0.00))))
                                                        .push(table_column(&format!("${:.2}", &item.net)))
                                                        .push(table_column(&item.client_name))
                                                        .push(table_column(&item.rep_name))
                                                        .push(table_column(&item.note.clone().unwrap_or("".to_string()))))
                                                    )
                                                .style(
                                                    table_row_style()
                                                    )
                                                )
                                                .style(CustomButtonStyle)
                                                .on_press(AppMessage::ViewSale(item.clone()))
                                                .into()
                                                )))))
                                                .direction(Direction::Horizontal(Properties::default())))
                                                .style(table_style())
                                                .max_height(500))).into())
                                                .into()
    }

    fn edit_view(&self) -> Option<Element<AppMessage>> {
        if self.edit_sale {
        Some(Container::new(
            Column::new()
                .push(Container::new(
                        Column::new()
                        .push(Text::new("Edit Sale".to_string())
                              .size(24)
                              .horizontal_alignment(Horizontal::Center)
                              .width(Length::Fill))
                        .push(edit_column("Discount", &self.sale_to_edit.discount.unwrap_or(0.00).to_string(), |input| AppMessage::Sale(SaleMessage::DiscountInput(input, true))))
                        .push(edit_column("Date", &self.sale_to_edit.date, |input| AppMessage::Sale(SaleMessage::DateInput(input, true))))
                        .push(edit_column("Note", &self.sale_to_edit.note.clone().unwrap_or("".to_string()), |input| AppMessage::Sale(SaleMessage::NoteInput(input, true))))
                        .push(Row::new()
                              .push(Button::new(Text::new("Submit".to_string())
                                                .horizontal_alignment(Horizontal::Center))
                                    .on_press(AppMessage::Sale(SaleMessage::Submit(true)))
                                    .style(CustomMainButtonStyle)
                                    .width(Length::Fill))
                              .push(Column::new()
                                    .width(Length::Fill))
                              .push(Button::new(Text::new("Delete".to_string())
                                                .horizontal_alignment(Horizontal::Center))
                                    .on_press(AppMessage::Sale(SaleMessage::Delete))
                                    .width(Length::Fill).style(iced::theme::Button::Destructive))))
                    .padding(24)
            ).max_width(700))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
            .into())
        } else {
            None
        }
    }

    pub fn create_view(&self) -> Option<Element<AppMessage>> {
        if self.add_sale {
            Some(Scrollable::new(
                    Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
                    .push(Text::new("Add Sale".to_string())
                          .size(24))
                    .push(
                        Column::new()
                        .spacing(12)
                        .padding(24)
                        .max_width(1000)
                        .align_items(Alignment::Center)
                        .push(
                            text_input_column(
                                "Discount", 
                                &self.add_sales.discount.unwrap_or(0.00).to_string(),
                                |input| AppMessage::Sale(SaleMessage::DiscountInput(input, false)))
                            )
                        .push(
                            text_input_column(
                                "Date",
                                &self.add_sales.date,
                                |input| AppMessage::Sale(SaleMessage::DateInput(input, false))
                                )
                            )
                        .push(Column::new()
                              .padding([8, 0, 8, 0])
                              .width(Length::Fill)
                              .align_items(Alignment::Center)
                              .push(
                                  Row::new()
                                  .spacing(12)
                                  .push(self.select_product())
                                  .push(self.selected_products())
                                  )
                             )
                        .push(Column::new()
                              .padding([8, 0, 8, 0])
                              .width(Length::Fill)
                              .align_items(Alignment::Center)
                              .push(
                                  Row::new()
                                  .spacing(12)
                                  .push(self.select_client())
                                  .push(self.selected_client())
                                  )
                             )
                        .push(Column::new()
                              .padding([8, 0, 8, 0])
                              .width(Length::Fill)
                              .align_items(Alignment::Center)
                              .push(
                                  Row::new()
                                  .spacing(12)
                                  .push(self.select_rep())
                                  .push(self.selected_rep())
                                  )
                             )
                        .push(
                            text_input_column(
                                "Notes",
                                &self.add_sales.note.clone().unwrap_or("".to_string()),
                                |input| AppMessage::Sale(SaleMessage::NoteInput(input, false)))
                            )
                        .push(Button::new("Submit")
                              .on_press(AppMessage::Sale(SaleMessage::Submit(false)))
                              .style(CustomMainButtonStyle))))
                              .into())
        } else {
            None
        }
    }
    
    fn create_rep_view(&self) -> Option<Element<AppMessage>> {
        if self.create_rep {
            Some(Container::new(
                    Scrollable::new(
                    Column::new()
                    .push(edit_column("Name", &self.rep_to_create.name, |input| AppMessage::Sale(SaleMessage::RepName(input))))
                    .push(edit_column("Percentage", &self.rep_to_create.percentage.to_string(), |input| AppMessage::Sale(SaleMessage::RepPercentage(input))))
                    .push(Button::new("Submit")
                          .on_press(AppMessage::Sale(SaleMessage::CreateRepSubmit))
                          .style(CustomMainButtonStyle)))).into())
        } else {
            None
        }
    }

    fn create_client_view(&self) -> Option<Element<AppMessage>> {
        if self.create_client {
            Some(Container::new(
                    Scrollable::new(
                    Column::new()
                    .push(edit_column("Name", &self.client_to_create.name, |input| AppMessage::Sale(SaleMessage::ClientName(input))))
                    .push(edit_column("Address", &self.client_to_create.address, |input| AppMessage::Sale(SaleMessage::ClientAddress(input))))
                    .push(edit_column("Email", &self.client_to_create.email.clone().unwrap_or("".to_string()), |input| AppMessage::Sale(SaleMessage::ClientEmail(input))))
                    .push(Button::new("Submit")
                          .on_press(AppMessage::Sale(SaleMessage::CreateClientSubmit))
                          .style(CustomMainButtonStyle)
                          ))).into())
        } else {
            None
        }
    }

    fn view_sale(&self) -> Option<Element<AppMessage>> {
        if self.view_sale {
            Some(Container::new(
                    Column::new()
                            .push(
                                close_button(AppMessage::Sale(SaleMessage::CloseSale))
                                )
                    .push(
                        Row::new()
                        .spacing(12)
                        .push(
                            Column::new()
                            .push(
                                Row::new()
                                .push(
                                    Text::new("Status: ")
                                    )
                                .push(
                                Text::new(&self.sale_to_view.status)
                                )
                                )
                            )
                        .push(
                            Column::new()
                            .push(
                                Button::new("Complete")
                                .on_press(AppMessage::Sale(SaleMessage::Fulfill))
                                .style(CustomMainButtonStyle)
                                )
                            )
                        )
                    .push(
                    Row::new()
                    .push(Column::new()
                          .push(Row::new()
                                .push(Text::new("Sale")))
                          .push(Row::new()
                                .push(Text::new(&self.sale_to_view.date)))
                          .push(Row::new()
                                .push(Text::new("Products")))
                          .push(Column::new()
                                .extend(self.sale_products_to_view
                                        .iter()
                                        .map(|item|
                                             Row::new()
                                             .push(item_view(&item))
                                             .padding([8,0,8,0])
                                             .into()
                                            )))
                          .padding([0,12,0,0]))
                    .push(client_view(&self.client_to_view))
                    )).into())
        } else {
            None
        }
    }
}

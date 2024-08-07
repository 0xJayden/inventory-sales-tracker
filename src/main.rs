#![windows_subsystem = "windows"]
#![cfg_attr(
    all(windows, target_arch = "x86_64"),
    link_args = "-Wl,--subsystem,windows:6.0 /ENTRY:mainCRTStartup"
)]
#![cfg_attr(
    all(windows, target_arch = "x86"),
    link_args = "-Wl,--subsystem,windows:6.0 /ENTRY:mainCRTStartup"
)]

use std::env;

use error::Errorr;
use home::{get_home, HomeMessage, HomeState, SPS};
use iced::{
    executor, window, Application, Command, Element, Theme
};

use clients::{add_client, delete_client, edit_client, get_clients, Client, ClientMessage, ClientState};
use manufacture::{
    get_manufactures, Manufacture, ManufactureMessage, ManufactureState, ProductToSelect,
};
use parts::{get_parts, Part, PartToAdd, PartsMessage, PartsState};
use product::{
    get_product_parts, get_products, Product, ProductMessage, ProductPart, ProductState,
    ProductToAdd,
};
use purchase::{
    get_purchase_parts, get_purchases, PartToSelect, Purchase, PurchaseMessage, PurchasePart,
    PurchaseState, PurchaseToAdd,
};
use rep::{add_rep, delete_rep, edit_rep, get_reps, Rep, RepMessage, RepState};
use sales::{
    add_client_set, add_rep_set, get_products_and_clients, get_sale_products_and_client, get_sales, Sale, SaleMessage, SaleProductToAdd, SalesState, PCR, SC, R
};

mod clients;
mod error;
mod manufacture;
mod parts;
mod product;
mod purchase;
mod sales;
mod rep;
mod home;
mod components;

#[derive(Debug, Clone)]
pub enum AppMessage {
    Product(ProductMessage),
    GoToProducts,
    EditProduct(Product),
    ViewProduct(Product),
    SaveProductParts(Result<Vec<ProductPart>, Errorr>),
    RefetchProducts(Result<(), Errorr>),
    Sale(SaleMessage),
    GoToSales,
    EditSale(Sale),
    ViewSale(Sale),
    RefetchSales(Result<(), Errorr>),
    RefetchSalesAndSale(Result<(), Errorr>),
    SaveProductsAndClients(Result<PCR, Errorr>),
    Parts(PartsMessage),
    GoToParts,
    EditPart(Part),
    SaveParts(Result<Vec<Part>, Errorr>),
    RefetchParts(Result<(), Errorr>),
    Client(ClientMessage),
    GoToClients,
    EditClient(Client),
    RefetchClients(Result<(), Errorr>),
    SaveClients(Result<Vec<Client>, Errorr>),
    Purchase(PurchaseMessage),
    GoToPurchases,
    ViewPurchase(Purchase),
    SavePurchasePartsToView(Result<Vec<PurchasePart>, Errorr>),
    EditPurchase(Purchase),
    SavePurchases(Result<Vec<Purchase>, Errorr>),
    SavePurchaseParts(Result<Vec<Part>, Errorr>),
    RefetchPurchases(Result<(), Errorr>),
    Manufacture(ManufactureMessage),
    GoToManufactures,
    EditManufacture(Manufacture),
    SaveManufactures(Result<Vec<Manufacture>, Errorr>),
    RefetchManufactures(Result<(), Errorr>),
    SaveManufactureProducts(Result<Vec<Product>, Errorr>),
    Rep(RepMessage),
    GoToReps,
    EditRep(Rep),
    SaveReps(Result<Vec<Rep>, Errorr>),
    SetRep(Result<R, Errorr>),
    RefetchReps(Result<(), Errorr>),
    SaveSaleProducts(Result<SC, Errorr>),
    SavePartsProducts(Result<Vec<Part>, Errorr>),
    DoIt(Result<(), Errorr>),
    SaveSales(Result<Vec<Sale>, Errorr>),
    SaveSalesAndSale(Result<Vec<Sale>, Errorr>),
    SaveProducts(Result<Vec<Product>, Errorr>),
    SetClientId(Result<i64, Errorr>),
    RefetchPurchaseParts(Result<(), Errorr>),
    GoToHome,
    Home(HomeMessage),
    SaveHome(Result<SPS, Errorr>)
}

#[derive(Default, Clone)]
pub struct App {
    show_products: bool,
    show_sales: bool,
    show_clients: bool,
    show_parts: bool,
    show_purchases: bool,
    show_manufactures: bool,
    show_reps: bool,
    pub sales: SalesState,
    pub products: ProductState,
    pub parts: PartsState,
    pub purchase: PurchaseState,
    pub clients: ClientState,
    pub reps: RepState,
    pub manufacture: ManufactureState,
    pub home: HomeState
}

impl App {
    fn clear_state(&mut self) {
        self.show_products = false;
        self.show_sales = false;
        self.show_clients = false;
        self.show_parts = false;
        self.show_purchases = false;
        self.show_manufactures = false;
        self.show_reps = false;
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = AppMessage;
    type Theme = Theme;

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (App::default(), Command::perform(get_home(), AppMessage::SaveHome))
    }

    fn title(&self) -> String {
        String::from("PHG")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::Home(msg) => {
                let _ = self.home.update(msg.clone());

                Command::none()
            }
            AppMessage::SaveHome(r) => {
                match r {
                    Ok(x) => {
                        self.home.sales = x.sales;
                        self.home.products = x.products;
                        self.home.parts = x.parts;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::Product(msg) => {
                let _ = self.products.update(msg.clone());

                match msg {
                    ProductMessage::Submit(is_edit) => match is_edit {
                        true => {
                            let i = self.products.product_to_edit.clone();
                            Command::perform(
                                ProductState::edit_product(i),
                                AppMessage::RefetchProducts,
                            )
                        }
                        false => {
                            let product_to_add = self.products.product_to_add.clone();
                            let parts_to_add = self.products.filtered_parts.clone();
                            Command::perform(
                                ProductState::add_product(product_to_add, parts_to_add),
                                AppMessage::RefetchProducts,
                            )
                        }
                    },
                    ProductMessage::ShowAddProduct => {
                        Command::perform(get_parts(), AppMessage::SavePartsProducts)
                    }
                    ProductMessage::Delete => {
                        let i = self.products.product_to_edit.clone();
                        Command::perform(ProductState::delete_product(i), AppMessage::DoIt)
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::Sale(msg) => {
                let _ = self.sales.update(msg.clone());

                match msg {
                    SaleMessage::Submit(is_edit) => {
                        if is_edit {
                            let i = self.sales.sale_to_edit.clone();
                            Command::perform(SalesState::edit_sale(i), AppMessage::DoIt)
                        } else {
                            let i = self.sales.add_sales.clone();
                            let j = self.sales.products_to_select.clone();
                            let k = self.sales.products_to_add.clone();
                            Command::perform(SalesState::add_sales(j, k, i), AppMessage::RefetchSales)
                        }
                    }
                    SaleMessage::Delete => {
                        let i = self.sales.sale_to_edit.clone();
                        Command::perform(SalesState::delete_sale(i), AppMessage::DoIt)
                    }
                    SaleMessage::ShowAddProducts => Command::perform(
                        get_products_and_clients(),
                        AppMessage::SaveProductsAndClients,
                    ),
                    SaleMessage::CreateClientSubmit => {
                        let c = self.sales.client_to_create.clone();
                        Command::perform(
                            add_client_set(c),
                            AppMessage::SetClientId,
                        )
                    }
                    SaleMessage::CreateRepSubmit => {
                        let c = self.sales.rep_to_create.clone();
                        Command::perform(
                            add_rep_set(c),
                            AppMessage::SetRep,
                        )
                    }
                    SaleMessage::Fulfill => {
                        Command::perform(SalesState::fulfill_sale(self.sales.sale_to_view.sale_id), AppMessage::RefetchSalesAndSale)
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::Purchase(msg) => {
                let _ = self.purchase.update(msg.clone());
                match msg {
                    PurchaseMessage::ShowAddPurchase => {
                        Command::perform(get_parts(), AppMessage::SavePurchaseParts)
                    }
                    PurchaseMessage::CreatePartSubmit => {
                        let p = &self.purchase.part_to_create.name;

                        let pp = PartToAdd {
                            name: p.to_string(),
                        };

                        Command::perform(PartsState::add_part(pp), AppMessage::RefetchPurchaseParts)
                    }
                    PurchaseMessage::Submit => {
                        let parts_to_add = self.purchase.parts_to_add.clone();
                        let purchase_to_add = self.purchase.purchase_to_add.clone();
                        let parts = self.purchase.parts.clone();
                        Command::perform(
                            PurchaseState::add_purchase(parts_to_add, purchase_to_add, parts),
                            AppMessage::RefetchPurchases,
                        )
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::Parts(msg) => {
                let _ = self.parts.update(msg.clone());
                match msg {
                    PartsMessage::Submit(is_edit) => {
                        if is_edit {
                            let p = self.parts.part_to_edit.clone();
                            Command::perform(PartsState::edit_part(p), AppMessage::RefetchParts)
                        } else {
                            let p = self.parts.part_to_add.clone();
                            Command::perform(PartsState::add_part(p), AppMessage::RefetchParts)
                        }
                    }
                    PartsMessage::Delete => {
                        let p = self.parts.part_to_edit.clone();
                        Command::perform(PartsState::delete_part(p), AppMessage::RefetchParts)
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::Client(msg) => {
                let _ = self.clients.update(msg.clone());
                match msg {
                    ClientMessage::Submit(is_edit) => {
                        if is_edit {
                            let p = self.clients.client_to_edit.clone();
                            Command::perform(edit_client(p), AppMessage::RefetchClients)
                        } else {
                            let p = self.clients.client_to_add.clone();
                            Command::perform(add_client(p), AppMessage::RefetchClients)
                        }
                    }
                    ClientMessage::Delete => {
                        let p = self.clients.client_to_edit.clone();
                        Command::perform(delete_client(p), AppMessage::RefetchClients)
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::Rep(msg) => {
                let _ = self.reps.update(msg.clone());

                match msg {
                    RepMessage::Submit(is_edit) => {
                        if is_edit {
                            let p = self.reps.rep_to_edit.clone();
                            Command::perform(edit_rep(p), AppMessage::RefetchReps)
                        } else {
                            let p = self.reps.rep_to_add.clone();
                            Command::perform(add_rep(p), AppMessage::RefetchReps)
                        }
                    }
                    RepMessage::Delete => {
                        let p = self.reps.rep_to_edit.clone();
                        Command::perform(delete_rep(p), AppMessage::RefetchReps)
                    }
                    _ => Command::none(),
                }
            }
            AppMessage::GoToReps => {
                self.clear_state();
                self.show_reps = true;
                Command::perform(get_reps(), AppMessage::SaveReps)
            }
            AppMessage::EditRep(rep) => {
                self.reps.rep_to_edit = rep;
                self.reps.edit_rep = true;
                Command::none()
            }
            AppMessage::SaveReps(r) => {
                match r {
                    Ok(i) => {
                        self.reps.reps = i;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::ViewPurchase(p) => {
                self.purchase.view_purchase = true;
                self.purchase.purchase_to_view = p.clone();
                Command::perform(
                    get_purchase_parts(p.id),
                    AppMessage::SavePurchasePartsToView,
                )
            }
            AppMessage::SavePurchasePartsToView(r) => {
                match r {
                    Ok(pp) => self.purchase.purchase_parts_to_view = pp,
                    Err(_) => println!("error"),
                }
                Command::none()
            }
            AppMessage::Manufacture(msg) => {
                let _ = self.manufacture.update(msg.clone());

                match msg {
                    ManufactureMessage::ShowAddManufacture => {
                        Command::perform(get_products(), AppMessage::SaveManufactureProducts)
                    }
                    ManufactureMessage::Submit(is_edit) => match is_edit {
                        true => {
                            let i = self.manufacture.manufacture_to_edit.clone();
                            Command::perform(
                                ManufactureState::edit_manufacture(i),
                                AppMessage::RefetchManufactures,
                            )
                        }
                        false => {
                            let products_to_add = self.manufacture.products_to_add.clone();
                            let manufacture_to_add = self.manufacture.manufacture_to_add.clone();
                            let products = self.manufacture.products.clone();
                            Command::perform(
                                ManufactureState::add_manufacture(
                                    products_to_add,
                                    manufacture_to_add,
                                    products,
                                ),
                                AppMessage::RefetchManufactures,
                            )
                        }
                    },
                    _ => Command::none(),
                }
            }
            AppMessage::GoToProducts => {
                self.clear_state();
                self.show_products = true;
                Command::perform(get_products(), AppMessage::SaveProducts)
            }
            AppMessage::ViewProduct(p) => {
                self.products.product_to_view = p.clone();
                self.products.view_product = true;
                Command::perform(
                    get_product_parts(p.product_id),
                    AppMessage::SaveProductParts,
                )
            }
            AppMessage::SaveProductParts(r) => {
                match r {
                    Ok(p) => {
                        self.products.product_parts_to_view = p;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::EditProduct(product) => {
                self.products.product_to_edit = product;
                self.products.edit_product = true;
                Command::none()
            }
            AppMessage::GoToSales => {
                self.clear_state();

                self.show_sales = true;
                Command::perform(get_sales(), AppMessage::SaveSales)
            }
            AppMessage::GoToHome => {
                self.clear_state();
                Command::perform(get_home(), AppMessage::SaveHome)
            }
            AppMessage::GoToClients => {
                self.clear_state();
                self.show_clients = true;
                Command::perform(get_clients(), AppMessage::SaveClients)
            }
            AppMessage::EditSale(s) => {
                self.sales.sale_to_edit = s;
                self.sales.edit_sale = true;
                Command::none()
            }
            AppMessage::ViewSale(s) => {
                self.sales.sale_to_view = s.clone();
                self.sales.view_sale = true;
                Command::perform(get_sale_products_and_client(s.sale_id, s.client_id), AppMessage::SaveSaleProducts)
            }
            AppMessage::EditClient(c) => {
                self.clients.client_to_edit = c;
                self.clients.edit_client = true;
                Command::none()
            }
            AppMessage::GoToParts => {
                self.clear_state();
                self.show_parts = true;
                Command::perform(get_parts(), AppMessage::SaveParts)
            }
            AppMessage::EditPart(p) => {
                self.parts.part_to_edit = p;
                self.parts.edit_part = true;
                Command::none()
            }
            AppMessage::GoToPurchases => {
                self.clear_state();
                self.show_purchases = true;
                Command::perform(get_purchases(), AppMessage::SavePurchases)
            }
            AppMessage::EditPurchase(p) => {
                self.purchase.purchase_to_edit = p;
                self.purchase.edit_purchase = true;
                Command::none()
            }
            AppMessage::GoToManufactures => {
                self.clear_state();
                self.show_manufactures = true;
                Command::perform(get_manufactures(), AppMessage::SaveManufactures)
            }
            AppMessage::EditManufacture(m) => {
                self.manufacture.manufacture_to_edit = m;
                self.manufacture.edit_manufacture = true;
                Command::none()
            }
            AppMessage::DoIt(r) => {
                match r {
                    Ok(()) => {
                        self.sales.products_to_add = Vec::new();
                        self.sales.add_sales = sales::Sale::default();
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveSales(r) => {
                match r {
                    Ok(s) => {
                        self.sales.sales = s;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveSalesAndSale(r) => {
                match r {
                    Ok(s) => {
                        self.sales.sales = s;
                        self.sales.sale_to_view.status = "COMPLETED".to_string();
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveProducts(r) => {
                match r {
                    Ok(i) => {
                        self.products.products = i;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveClients(r) => {
                match r {
                    Ok(clients) => {
                        self.clients.clients = clients.clone();
                        self.sales.clients = clients;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveSaleProducts(r) => {
                match r {
                    Ok(s) => {
                        self.sales.sale_products_to_view = s.sale_products;
                        self.sales.client_to_view = s.client;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SetClientId(r) => {
                match r {
                    Ok(i) => {
                        self.sales.add_sales.client_id = i;
                        Command::perform(get_products_and_clients(), AppMessage::SaveProductsAndClients)
                    }
                    Err(_) => {
                        println!("error");
                        Command::none()
                    }
                }
            }
            AppMessage::SetRep(r) => {
                match r {
                    Ok(i) => {
                        self.sales.add_sales.rep_id = Some(i.id);
                        self.sales.add_sales.rep_name = i.name;
                        Command::perform(get_products_and_clients(), AppMessage::SaveProductsAndClients)
                    }
                    Err(_) => {
                        println!("error");
                        Command::none()
                    }
                }
            }
            AppMessage::SaveParts(r) => {
                match r {
                    Ok(p) => {
                        self.parts.parts = p;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SavePartsProducts(r) => {
                match r {
                    Ok(p) => {
                        let mut x = Vec::new();

                        for part in p.iter() {
                            let part_product = PartToSelect {
                                part_id: part.part_id,
                                name: part.name.clone(),
                                cost: part.cost.to_string(),
                                qty: 0,
                                total_spent: part.total_spent,
                                total_units_purchased: part.total_units_purchased,
                            };

                            x.push(part_product);
                        }

                        self.products.parts = p;
                        self.products.parts_to_select = x.clone();
                        self.products.filtered_parts = x;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SavePurchases(r) => {
                match r {
                    Ok(p) => {
                        self.purchase.purchases = p;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SavePurchaseParts(r) => {
                match r {
                    Ok(p) => {
                        let mut x = Vec::new();

                        for part in p.iter() {
                            let part_to_select = PartToSelect {
                                part_id: part.part_id,
                                name: part.name.clone(),
                                cost: 0.00.to_string(),
                                qty: 0,
                                total_spent: part.total_spent,
                                total_units_purchased: part.total_units_purchased,
                            };

                            x.push(part_to_select);
                        }

                        self.purchase.parts = p;
                        self.purchase.parts_to_select = x.clone();
                        self.purchase.filtered_parts = x;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveManufactures(r) => {
                match r {
                    Ok(m) => {
                        self.manufacture.manufactures = m;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveManufactureProducts(r) => {
                match r {
                    Ok(p) => {
                        let mut x = Vec::new();

                        for product in p.iter() {
                            let product_to_select = ProductToSelect {
                                product_id: product.product_id,
                                name: product.name.clone(),
                                qty: 0,
                            };

                            x.push(product_to_select);
                        }

                        self.manufacture.products = p;
                        self.manufacture.products_to_select = x.clone();
                        self.manufacture.filtered_products = x;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::SaveProductsAndClients(r) => {
                match r {
                    Ok(pc) => {
                        self.sales.clients = pc.clients.clone();
                        self.sales.filtered_clients = pc.clients;
                        self.sales.reps = pc.reps.clone();
                        self.sales.filtered_reps = pc.reps;

                        let mut x: Vec<SaleProductToAdd> = Vec::new();

                        for p in &pc.products {
                            let ps = SaleProductToAdd {
                                product_id: p.product_id,
                                name: p.name.clone(),
                                cost: p.cost,
                                msrp: p.msrp,
                                units: p.units,
                                qty: 0,
                            };

                            x.push(ps);
                        }

                        self.sales.products_to_select = x.clone();
                        self.sales.filtered_products = x;
                    }
                    Err(_) => {
                        println!("error");
                    }
                }
                Command::none()
            }
            AppMessage::RefetchPurchaseParts(r) => match r {
                Ok(_) => Command::perform(get_parts(), AppMessage::SavePurchaseParts),
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchPurchases(r) => match r {
                Ok(_) => {
                    self.purchase.purchase_to_add = PurchaseToAdd::default();
                    self.purchase.parts_to_add = Vec::new();
                    Command::perform(get_purchases(), AppMessage::SavePurchases)
                }
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchSales(r) => match r {
                Ok(_) => {
                    Command::perform(get_sales(), AppMessage::SaveSales)
                }
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchSalesAndSale(r) => match r {
                Ok(_) => {
                    Command::perform(get_sales(), AppMessage::SaveSalesAndSale)
                }
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchReps(r) => match r {
                Ok(_) => Command::perform(get_reps(), AppMessage::SaveReps),
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchClients(r) => match r {
                Ok(_) => Command::perform(get_clients(), AppMessage::SaveClients),
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchParts(r) => match r {
                Ok(_) => Command::perform(get_parts(), AppMessage::SaveParts),
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchManufactures(r) => match r {
                Ok(_) => Command::perform(get_manufactures(), AppMessage::SaveManufactures),
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
            AppMessage::RefetchProducts(r) => match r {
                Ok(_) => {
                    self.products.product_to_add = ProductToAdd::default();
                    self.products.parts_to_add = Vec::new();
                    Command::perform(get_products(), AppMessage::SaveProducts)
                }
                Err(_) => {
                    println!("error");
                    Command::none()
                }
            },
        }
    }

    fn view(&self) -> Element<Self::Message> {
        if self.show_products {
            ProductState::view(&self.products)
        } else if self.show_sales {
            SalesState::view(&self.sales)
        } else if self.show_clients {
            ClientState::view(&self.clients)
        } else if self.show_parts {
            PartsState::view(&self.parts)
        } else if self.show_purchases {
            PurchaseState::view(&self.purchase)
        } else if self.show_manufactures {
            ManufactureState::view(&self.manufacture)
        } else if self.show_reps {
            RepState::view(&self.reps)
        } else {
            HomeState::view(&self.home)
        }
    }
}


#[tokio::main]
async fn main() -> iced::Result {
    App::run(
        iced::Settings::from(
            iced::Settings {
                window: window::Settings {
                    ..Default::default()
                },
                ..Default::default()
            }
            )
        )
}

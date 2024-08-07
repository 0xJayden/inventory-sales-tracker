-- Add migration script here
CREATE TABLE Client (
    client_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    email TEXT
);

CREATE TABLE Product (
    product_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    units INTEGER DEFAULT 0 NOT NULL,
    cost REAL DEFAULT 0.00 NOT NULL,
    msrp  REAL DEFAULT 0.00 NOT NULL
);

CREATE TABLE Part (
    part_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cost REAL NOT NULL DEFAULT 0.00,
    total_spent REAL NOT NULL DEFAULT 0.00,
    total_units_purchased INTEGER NOT NULL DEFAULT 0,
    units_left INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE ProductPart (
    product_part_id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost REAL NOT NULL, 
    product_id INTEGER NOT NULL, 
    part_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id),
    FOREIGN KEY (part_id) REFERENCES Part (part_id)
);

CREATE TABLE Purchase (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
    total REAL NOT NULL,
    note TEXT
);

CREATE TABLE PurchasePart (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost REAL NOT NULL,
    purchase_id INTEGER NOT NULL,
    part_id INTEGER NOT NULL,
    FOREIGN KEY (purchase_id) REFERENCES Purchase (id),
    FOREIGN KEY (part_id) REFERENCES Part (part_id)
);

CREATE TABLE Sale (
    sale_id INTEGER PRIMARY KEY,
    discount REAL,
    total REAL NOT NULL,
    cost REAL NOT NULL,
    net REAL NOT NULL,
    date TEXT NOT NULL,
    note TEXT,
    client_id INTEGER NOT NULL,
    FOREIGN KEY (client_id) REFERENCES Client (client_id)
);

CREATE TABLE SaleProduct (
    sale_product_id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost_at_sale REAL NOT NULL,
    msrp_at_sale REAL NOT NULL,
    product_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id),
    FOREIGN KEY (sale_id) REFERENCES Sale (sale_id)
);

CREATE TABLE Manufacture (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL
);

CREATE TABLE ManufactureProduct (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    manufacture_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id),
    FOREIGN KEY (manufacture_id) REFERENCES Manufacture (id)
);

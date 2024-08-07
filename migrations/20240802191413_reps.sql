-- Add migration script here
CREATE TABLE Rep (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    percentage INTEGER NOT NULL
);

CREATE TABLE NewSale (
    sale_id INTEGER PRIMARY KEY,
    discount REAL,
    total REAL NOT NULL,
    cost REAL NOT NULL,
    net REAL NOT NULL,
    date TEXT NOT NULL,
    note TEXT,
    client_id INTEGER NOT NULL,
    rep_id INTEGER,
    FOREIGN KEY (rep_id) REFERENCES Rep (id), 
    FOREIGN KEY (client_id) REFERENCES Client (client_id)
);

INSERT INTO NewSale SELECT *, NULL FROM Sale;

CREATE TABLE SaleProductt (
    sale_product_id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost_at_sale REAL NOT NULL,
    msrp_at_sale REAL NOT NULL,
    product_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id),
    FOREIGN KEY (sale_id) REFERENCES NewSale (sale_id)
);

INSERT INTO SaleProductt SELECT * FROM SaleProduct;

DROP TABLE SaleProduct;

DROP TABLE Sale;

ALTER TABLE NewSale RENAME TO Sale;

ALTER TABLE SaleProductt RENAME TO SaleProduct;

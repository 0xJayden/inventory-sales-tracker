-- Add migration script here
PRAGMA foreign_keys = OFF;

CREATE TABLE SaleProductNew (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost_at_sale REAL NOT NULL,
    msrp_at_sale REAL NOT NULL,
    product_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id) ON DELETE CASCADE,
    FOREIGN KEY (sale_id) REFERENCES Sale (id) ON DELETE CASCADE
);

INSERT INTO SaleProductNew SELECT * FROM SaleProduct;

DROP TABLE SaleProduct;

ALTER TABLE SaleProductNew RENAME TO SaleProduct;

PRAGMA foreign_keys = ON;

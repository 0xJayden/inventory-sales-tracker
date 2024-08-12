-- Add migration script here
CREATE TEMPORARY TABLE T AS SELECT * FROM SaleProduct;
CREATE TEMPORARY TABLE U AS SELECT * FROM ManufactureProduct;

DROP TABLE SaleProduct;
DROP TABLE ManufactureProduct;

CREATE TABLE SaleProduct (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost_at_sale REAL NOT NULL,
    msrp_at_sale REAL NOT NULL,
    product_id INTEGER NOT NULL,
    sale_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id) ON DELETE CASCADE,
    FOREIGN KEY (sale_id) REFERENCES Sale (sale_id) ON DELETE CASCADE
);

CREATE TABLE ManufactureProduct (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    manufacture_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id) ON DELETE CASCADE,
    FOREIGN KEY (manufacture_id) REFERENCES Manufacture (id) ON DELETE CASCADE
);

INSERT INTO SaleProduct (id, qty, cost_at_sale, msrp_at_sale, product_id, sale_id) SELECT * FROM T;
INSERT INTO ManufactureProduct (id, qty, product_id, manufacture_id) SELECT * FROM U;

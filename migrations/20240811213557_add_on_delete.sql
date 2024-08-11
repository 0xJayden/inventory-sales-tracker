-- Add migration script here
CREATE TEMPORARY TABLE T AS SELECT * FROM ProductPart;

DROP TABLE ProductPart;

CREATE TABLE ProductPart (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost REAL NOT NULL, 
    product_id INTEGER NOT NULL, 
    part_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product (product_id) ON DELETE CASCADE,
    FOREIGN KEY (part_id) REFERENCES Part (part_id) ON DELETE CASCADE
);

INSERT INTO ProductPart (id, qty, cost, product_id, part_id) SELECT product_part_id, qty, cost, product_id, part_id FROM T;

DROP TABLE T;

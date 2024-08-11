-- Add migration script here
CREATE TEMPORARY TABLE T AS SELECT * FROM PurchasePart;

DROP TABLE PurchasePart;

CREATE TABLE PurchasePart (
    id INTEGER PRIMARY KEY,
    qty INTEGER NOT NULL,
    cost REAL NOT NULL, 
    purchase_id INTEGER NOT NULL, 
    part_id INTEGER NOT NULL,
    FOREIGN KEY (purchase_id) REFERENCES Purchase (id) ON DELETE CASCADE,
    FOREIGN KEY (part_id) REFERENCES Part (part_id) ON DELETE CASCADE
);

INSERT INTO PurchasePart (id, qty, cost, purchase_id, part_id) SELECT * FROM T;

DROP TABLE T;

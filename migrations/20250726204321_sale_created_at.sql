-- Add migration script here
CREATE TABLE SaleNew (
    id INTEGER PRIMARY KEY,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    discount REAL,
    total REAL NOT NULL,
    cost REAL NOT NULL,
    net REAL NOT NULL,
    date TEXT NOT NULL,
    note TEXT,
    client_id INTEGER NOT NULL,
    rep_id INTEGER, shipping REAL DEFAULT 15.00 NOT NULL, status TEXT DEFAULT "DRAFT" NOT NULL, rep_cut REAL,
    FOREIGN KEY (rep_id) REFERENCES Rep (id), 
    FOREIGN KEY (client_id) REFERENCES Client (client_id)
);

INSERT INTO SaleNew (id, created_at, discount, total, cost, net, date, note, client_id, rep_id, shipping, status, rep_cut) 
SELECT sale_id, datetime('now'), discount, total, cost, net, date, note, client_id, rep_id, shipping, status, rep_cut 
FROM Sale;

DROP TABLE Sale;

ALTER TABLE SaleNew RENAME TO Sale

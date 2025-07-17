-- Add migration script here
ALTER TABLE ProductPart ADD COLUMN new_qty REAL NOT NULL DEFAULT 0.00;

UPDATE ProductPart SET new_qty = qty;

ALTER TABLE ProductPart DROP COLUMN qty;

ALTER TABLE ProductPart RENAME COLUMN new_qty TO qty;

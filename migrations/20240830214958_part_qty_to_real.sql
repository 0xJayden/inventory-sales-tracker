-- Add migration script here
ALTER TABLE Part ADD COLUMN new_qty REAL NOT NULL DEFAULT 0.00;
ALTER TABLE Part ADD COLUMN new_qty2 REAL NOT NULL DEFAULT 0.00;

UPDATE Part SET new_qty = total_units_purchased;
UPDATE Part SET new_qty2 = units_left;

ALTER TABLE Part DROP COLUMN total_units_purchased;
ALTER TABLE Part DROP COLUMN units_left;

ALTER TABLE Part RENAME COLUMN new_qty TO total_units_purchased;
ALTER TABLE Part RENAME COLUMN new_qty2 TO units_left;

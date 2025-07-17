-- Add migration script here
ALTER TABLE Purchase ADD COLUMN shipping REAL DEFAULT 0.00 NOT NULL;

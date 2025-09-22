CREATE TABLE coffee_order (
	id INTEGER NOT NULL PRIMARY KEY,
	created DATETIME NOT NULL DEFAULT (datetime('now'))
);

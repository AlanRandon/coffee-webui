CREATE TABLE coffee_order (
	id INTEGER PRIMARY KEY,
	created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	product INTEGER NOT NULL,
	FOREIGN KEY (product) REFERENCES product (id)
);

CREATE TABLE product (
	id INTEGER PRIMARY KEY,
	name TEXT NOT NULL
);

INSERT INTO product (id, name) VALUES (NULL, "Pumpkin Spice Latte");
INSERT INTO product (id, name) VALUES (NULL, "Yummy Blood");

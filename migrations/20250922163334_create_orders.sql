CREATE TABLE coffee_order (
	id INTEGER PRIMARY KEY,
	created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	price UNSIGNED INTEGER NOT NULL,
	product INTEGER NOT NULL,
	FOREIGN KEY (product) REFERENCES product (id)
);

CREATE TABLE product (
	id INTEGER PRIMARY KEY,
	name TEXT NOT NULL,
	current_price UNSIGNED INTEGER NOT NULL
);

INSERT INTO product (id, name, current_price) VALUES (NULL, "Pumpkin Spice Latte", 420);
INSERT INTO product (id, name, current_price) VALUES (NULL, "Yummy Blood", 690);

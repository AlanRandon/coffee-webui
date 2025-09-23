create-db:
	sqlx db create
	sqlx migrate run

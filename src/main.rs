use askama::Template;
use poem::{EndpointExt, http::StatusCode, middleware::AddData};
use serde::{Deserialize, de};
use sqlx::{SqlitePool, types::chrono};
use std::sync::Arc;

fn response(template: impl Template, status: StatusCode) -> poem::Response {
    match template.render() {
        Ok(body) => poem::Response::builder().status(status).body(body),
        Err(_) => poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(()),
    }
}

struct Product {
    id: i64,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
struct OrderRow {
    id: i64,
    created: chrono::NaiveDateTime,
    product_name: String,
}

#[derive(Template)]
#[template(path = "error.html")]
struct Error;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    orders: Vec<OrderRow>,
    products: Vec<Product>,
}

#[derive(Template)]
#[template(path = "index.html", block = "order_table")]
struct OrderTable {
    orders: Vec<OrderRow>,
}

async fn get_orders(pool: &SqlitePool) -> Result<Vec<OrderRow>, impl std::error::Error> {
    sqlx::query_as!(
        OrderRow,
        "SELECT coffee_order.id as id, created, product.name as product_name FROM coffee_order INNER JOIN product ON coffee_order.product = product.id ORDER BY created DESC"
    )
    .fetch_all(pool)
    .await
}

#[poem::handler]
async fn index(pool: poem::web::Data<&Arc<SqlitePool>>) -> poem::Response {
    let Ok(orders) = get_orders(&pool).await else {
        return response(Error, StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(products) = sqlx::query_as!(Product, "SELECT id, name FROM product")
        .fetch_all(pool.as_ref())
        .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    response(Index { orders, products }, StatusCode::OK)
}

#[derive(Deserialize)]
struct CreateRequest {
    product: i64,
}

#[poem::handler]
async fn create_order(
    pool: poem::web::Data<&Arc<SqlitePool>>,
    form: poem::web::Form<CreateRequest>,
) -> poem::Response {
    let Ok(_) = sqlx::query!(
        "INSERT INTO coffee_order (id, product, price) VALUES (NULL, ?, (SELECT current_price FROM product WHERE id = ?))",
        form.product,
        form.product
    )
    .execute(pool.as_ref())
    .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    let Ok(orders) = get_orders(&pool).await else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    response(OrderTable { orders }, StatusCode::OK)
}

#[derive(Deserialize, Debug)]
struct DeleteRequest {
    id: i64,
}

#[poem::handler]
async fn delete_order(
    pool: poem::web::Data<&Arc<SqlitePool>>,
    query: poem::web::Query<DeleteRequest>,
) -> poem::Response {
    let Ok(_) = sqlx::query!("DELETE FROM coffee_order WHERE id = (?)", query.id)
        .execute(pool.as_ref())
        .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    poem::Response::builder().status(StatusCode::OK).body(())
}

#[derive(Deserialize)]
struct CreateProductRequest {
    name: String,
    #[serde(deserialize_with = "deserialize_price")]
    price: u16,
}

fn deserialize_price<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: de::Deserializer<'de>,
{
    let price = String::deserialize(deserializer)?;
    if let Some((whole, fraction)) = price.split_once('.') {
        let whole = whole.parse::<u16>().map_err(de::Error::custom)?;
        let fraction = match fraction.len() {
            1 => 10,
            2 => 1,
            _ => return Err(de::Error::custom("invalid price fraction")),
        } * fraction.parse::<u16>().map_err(de::Error::custom)?;

        return whole
            .checked_mul(100)
            .and_then(|price| price.checked_add(fraction))
            .ok_or_else(|| de::Error::custom("price too large"));
    }

    price.parse::<u16>().map_err(de::Error::custom)
}

#[derive(Template)]
#[template(path = "index.html", block = "product_list")]
struct ProductList {
    products: Vec<Product>,
}

#[poem::handler]
async fn create_product(
    pool: poem::web::Data<&Arc<SqlitePool>>,
    form: poem::web::Form<CreateProductRequest>,
) -> poem::Response {
    let Ok(_) = sqlx::query!(
        "INSERT INTO product (id, name, current_price) VALUES (NULL, ?, ?)",
        form.name,
        form.price
    )
    .execute(pool.as_ref())
    .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    let Ok(products) = sqlx::query_as!(Product, "SELECT id, name FROM product")
        .fetch_all(pool.as_ref())
        .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    response(ProductList { products }, StatusCode::OK)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = Arc::new(SqlitePool::connect(&database_url).await?);

    sqlx::migrate!().run(pool.as_ref()).await?;

    let app = poem::Route::new()
        .at("/", poem::get(index))
        .at("/hx/create_order", poem::post(create_order))
        .at("/hx/create_product", poem::post(create_product))
        .at("/hx/delete_order", poem::delete(delete_order))
        .with(AddData::new(pool));

    let listener = poem::listener::TcpListener::bind("127.0.0.1:8000");
    Ok(poem::Server::new(listener).run(app).await?)
}

use askama::Template;
use poem::{EndpointExt, http::StatusCode, middleware::AddData};
use serde::Deserialize;
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
struct Order {
    id: i64,
    created: chrono::NaiveDateTime,
}

#[derive(Template)]
#[template(path = "error.html")]
struct Error;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    orders: Vec<Order>,
    products: Vec<Product>,
}

#[derive(Template)]
#[template(path = "index.html", block = "order_table")]
struct OrderTable {
    orders: Vec<Order>,
}

#[poem::handler]
async fn index(pool: poem::web::Data<&Arc<SqlitePool>>) -> poem::Response {
    let Ok(orders) = sqlx::query_as!(
        Order,
        "SELECT id, created FROM coffee_order ORDER BY created DESC"
    )
    .fetch_all(pool.as_ref())
    .await
    else {
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
    query: poem::web::Query<CreateRequest>,
) -> poem::Response {
    let Ok(_) = sqlx::query!(
        "INSERT INTO coffee_order (id, product) VALUES (NULL, ?)",
        query.product
    )
    .execute(pool.as_ref())
    .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    let Ok(orders) = sqlx::query_as!(
        Order,
        "SELECT id, created FROM coffee_order ORDER BY created DESC"
    )
    .fetch_all(pool.as_ref())
    .await
    else {
        return poem::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(());
    };

    response(OrderTable { orders, }, StatusCode::OK)
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = Arc::new(SqlitePool::connect(&database_url).await?);

    sqlx::migrate!().run(pool.as_ref()).await?;

    let app = poem::Route::new()
        .at("/", poem::get(index))
        .at("/hx/create_order", poem::post(create_order))
        .at("/hx/delete_order", poem::delete(delete_order))
        .with(AddData::new(pool));

    let listener = poem::listener::TcpListener::bind("127.0.0.1:3000");
    Ok(poem::Server::new(listener).run(app).await?)
}

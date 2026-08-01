use clickhouse::Client;
use dotenvy::dotenv;
use std::env;

pub fn create_clickhouse_client() -> Client {
    dotenv().ok();

    let url = env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
    let database = env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| "analytics".to_string());
    let user = env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string());
    let password = env::var("CLICKHOUSE_PASSWORD").unwrap_or_default();

    let mut client = Client::default()
        .with_url(url)
        .with_database(database)
        .with_user(user);

    if !password.is_empty() {
        client = client.with_password(password);
    }

    client
}
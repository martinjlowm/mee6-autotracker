use aws_sdk_dynamodb::Client as DynamoDBClient;
use tokio::sync::OnceCell;

pub const TABLE_NAME: &str = "autotracker-actions";

async fn dynamodb_client() -> DynamoDBClient {
    let config = aws_config::load_from_env().await;
    DynamoDBClient::new(&config)
}

pub static CLIENT: OnceCell<DynamoDBClient> = OnceCell::const_new();

pub async fn dynamodb<'client>() -> &'client DynamoDBClient {
    CLIENT.get_or_init(dynamodb_client).await
}

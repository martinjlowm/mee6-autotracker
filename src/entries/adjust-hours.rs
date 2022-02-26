use ::lib::services::dynamodb::dynamodb;
use anyhow::Result;
use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::prelude::Utc;
use jemallocator::Jemalloc;
use lambda_runtime::handler_fn;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

async fn handler(event: ApiGatewayProxyRequest, _: lambda_runtime::Context) -> Result<()> {
    dbg!(event.body);

    let dynamodb = dynamodb().await;

    let now = Utc::now().naive_utc();

    dynamodb
        .update_item()
        .table_name("autotracker-actions")
        .key(
            "pk",
            AttributeValue::S(format!("timestamp|{}", now.date().to_string().as_str())),
        )
        .key("sk", AttributeValue::S("void".to_string()))
        .expression_attribute_names("#hours", "hours")
        .expression_attribute_values(":hours", AttributeValue::N("8".to_string()))
        .update_expression("SET #hours = :hours")
        .expression_attribute_names("#pk", "pk")
        .condition_expression("attribute_exists(#pk)")
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    bb_rust::misc::setup_aws_lambda_logging();

    let res = lambda_runtime::run(handler_fn(handler)).await;

    if let Err(err) = res {
        log::error!("{:?}", err);
        std::process::exit(1);
    }
}

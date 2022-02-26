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

    let response = dynamodb
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
        .await;

    if let Err(sdk_err) = response {
        use aws_sdk_dynamodb::SdkError::*;

        match sdk_err {
            ConstructionFailure(_) => todo!(),
            TimeoutError(_) => todo!(),
            DispatchFailure(_) => todo!(),
            ResponseError {
                err: _err,
                raw: _raw,
            } => todo!(),
            ServiceError { err, raw: _raw } => {
                use aws_sdk_dynamodb::error::UpdateItemErrorKind::*;

                match err.kind {
                    ConditionalCheckFailedException(_) => {
                        log::info!("Conditional check failed - that's okay!");
                    }
                    InternalServerError(_) => todo!(),
                    InvalidEndpointException(_) => todo!(),
                    ItemCollectionSizeLimitExceededException(_) => todo!(),
                    ProvisionedThroughputExceededException(_) => todo!(),
                    RequestLimitExceeded(_) => todo!(),
                    ResourceNotFoundException(_) => todo!(),
                    TransactionConflictException(_) => todo!(),
                    Unhandled(_) => todo!(),
                    _ => todo!(),
                }
            }
        }
    }

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

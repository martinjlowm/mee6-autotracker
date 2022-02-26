use ::lib::services::dynamodb::dynamodb;
use ::lib::types::slack::Response;
use anyhow::{Context, Result};
use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::prelude::Utc;
use jemallocator::Jemalloc;
use lambda_runtime::handler_fn;
use serde_derive::{Deserialize, Serialize};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Deserialize, Serialize, Debug)]
pub struct SlackPayload {
    payload: String,
}

fn parse_slack_payload(body: &str) -> Result<Response> {
    let SlackPayload { payload } = serde_urlencoded::from_str(body)?;
    Ok(serde_json::from_str(payload.as_str())?)
}

async fn handler(event: ApiGatewayProxyRequest, _: lambda_runtime::Context) -> Result<()> {
    dbg!(event.clone());

    let payload = parse_slack_payload(event.body.with_context(|| "No body")?.as_str())?;

    let action = payload.actions.first().with_context(|| "No Slack action?")?;

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
        .expression_attribute_values(":hours", AttributeValue::N(action.text.text.to_string()))
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

#[cfg(test)]
mod tests {
    use super::parse_slack_payload;

    #[test]
    fn test_response_parsing() {
        let response = r#"payload=%7B%22type%22%3A%22block_actions%22%2C%22user%22%3A%7B%22id%22%3A%22U7XJ7HMEC%22%2C%22username%22%3A%22mj%22%2C%22name%22%3A%22mj%22%2C%22team_id%22%3A%22T7Z4HQ1L6%22%7D%2C%22api_app_id%22%3A%22A01G7GTKQKH%22%2C%22token%22%3A%22hModry2ZdOyl47cpLaiev1J7%22%2C%22container%22%3A%7B%22type%22%3A%22message%22%2C%22message_ts%22%3A%221645904837.581049%22%2C%22channel_id%22%3A%22D0341CNTLM8%22%2C%22is_ephemeral%22%3Afalse%7D%2C%22trigger_id%22%3A%223157103609190.271153817686.0189eef162c828c942ae6c6b5664e6b1%22%2C%22team%22%3A%7B%22id%22%3A%22T7Z4HQ1L6%22%2C%22domain%22%3A%22blackbird-crew%22%7D%2C%22enterprise%22%3Anull%2C%22is_enterprise_install%22%3Afalse%2C%22channel%22%3A%7B%22id%22%3A%22D0341CNTLM8%22%2C%22name%22%3A%22directmessage%22%7D%2C%22message%22%3A%7B%22bot_id%22%3A%22B03417WRY11%22%2C%22type%22%3A%22message%22%2C%22text%22%3A%22Should+I+adjust+the+number+of+hours+for+System+2+work%3F+You+have+until+end+of+day.%22%2C%22user%22%3A%22U03417K2FR8%22%2C%22ts%22%3A%221645904837.581049%22%2C%22team%22%3A%22T7Z4HQ1L6%22%2C%22blocks%22%3A%5B%7B%22type%22%3A%22section%22%2C%22block_id%22%3A%22l7%5C%2F%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%22Should+I+adjust+the+number+of+hours+for+System+2+work%3F+You+have+until+end+of+day.%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22actions%22%2C%22block_id%22%3A%22M%5C%2FE%22%2C%22elements%22%3A%5B%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%228zN%3D%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%220%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22Q3gd8%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%222%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22EiU%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%224%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22rtsA%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%226%22%2C%22emoji%22%3Afalse%7D%7D%5D%7D%5D%7D%2C%22state%22%3A%7B%22values%22%3A%7B%7D%7D%2C%22response_url%22%3A%22https%3A%5C%2F%5C%2Fhooks.slack.com%5C%2Factions%5C%2FT7Z4HQ1L6%5C%2F3163761419107%5C%2F8OI44EMzlWemCaoFurG2ch7m%22%2C%22actions%22%3A%5B%7B%22action_id%22%3A%22rtsA%22%2C%22block_id%22%3A%22M%5C%2FE%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%226%22%2C%22emoji%22%3Afalse%7D%2C%22type%22%3A%22button%22%2C%22action_ts%22%3A%221645904928.633379%22%7D%5D%7D"#;
        parse_slack_payload(response).unwrap();
    }
}

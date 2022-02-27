use ::lib::services::dynamodb::{dynamodb, TABLE_NAME};
use ::lib::types::slack::Response;
use anyhow::{anyhow, Context, Result};
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::prelude::{NaiveDateTime, Utc};
use chrono::Duration;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use jemallocator::Jemalloc;
use lambda_runtime::handler_fn;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use sha2::Sha256;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Deserialize, Serialize, Debug)]
pub struct SlackPayload {
    payload: String,
}

lazy_static! {
    static ref SLACK_SIGNING_SECRET: String =
        std::env::var("SLACK_SIGNING_SECRET").expect("SLACK_SIGNING_SECRET is not set!");
}

type HmacSha256 = Hmac<Sha256>;

// NOTE: Custom authorizers don't have access to body which is why we validate
// signature here :(
fn validate_signature(
    signing_secret: &str,
    request_timestamp: &str,
    signature: &str,
    body: &str,
) -> Result<()> {
    let sig_basestring = format!(
        "v0:{timestamp}:{body}",
        timestamp = request_timestamp,
        body = body
    );

    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes())?;
    mac.update(sig_basestring.as_bytes());

    let true_signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

    if true_signature.as_str() != signature {
        return Err(anyhow!("Invalid signature"));
    }

    Ok(())
}

fn parse_slack_payload(body: &str) -> Result<Response> {
    let SlackPayload { payload } = serde_urlencoded::from_str(body)?;
    Ok(serde_json::from_str(payload.as_str())?)
}

async fn handler(
    event: ApiGatewayProxyRequest,
    _: lambda_runtime::Context,
) -> Result<ApiGatewayProxyResponse> {
    let request_timestamp = event
        .headers
        .get("x-slack-request-timestamp")
        .with_context(|| "Missing x-slack-request-timestamp")?
        .to_str()?;

    let now = Utc::now().naive_utc();
    let timestamp = NaiveDateTime::from_timestamp(request_timestamp.parse::<i64>()?, 0);

    if Duration::minutes(5) < now - timestamp {
        return Err(anyhow!("Possible replay attack"));
    }

    let signature = event
        .headers
        .get("x-slack-signature")
        .with_context(|| "Missing x-slack-signature")?
        .to_str()?;

    let body = event.body.with_context(|| "No body")?;

    // FIXME: Handle signature validation failures more gracefully - we want to
    // propagate 4XX and 5XX errors, the latter is the current behavior.
    validate_signature(
        SLACK_SIGNING_SECRET.as_str(),
        request_timestamp,
        signature,
        body.as_str(),
    )?;

    let payload = parse_slack_payload(body.as_str())?;

    let action = payload
        .actions
        .first()
        .with_context(|| "No Slack action?")?;

    let dynamodb = dynamodb().await;

    let now = Utc::now().naive_utc();

    let response = dynamodb
        .update_item()
        .table_name(TABLE_NAME)
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

    let headers = HeaderMap::new();

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        headers: headers.clone(),
        multi_value_headers: headers,
        body: None,
        is_base64_encoded: None,
    })
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
    use super::{parse_slack_payload, validate_signature};

    #[test]
    fn test_response_parsing() {
        let response = r#"payload=%7B%22type%22%3A%22block_actions%22%2C%22user%22%3A%7B%22id%22%3A%22U7XJ7HMEC%22%2C%22username%22%3A%22mj%22%2C%22name%22%3A%22mj%22%2C%22team_id%22%3A%22T7Z4HQ1L6%22%7D%2C%22api_app_id%22%3A%22A01G7GTKQKH%22%2C%22token%22%3A%22hModry2ZdOyl47cpLaiev1J7%22%2C%22container%22%3A%7B%22type%22%3A%22message%22%2C%22message_ts%22%3A%221645904837.581049%22%2C%22channel_id%22%3A%22D0341CNTLM8%22%2C%22is_ephemeral%22%3Afalse%7D%2C%22trigger_id%22%3A%223157103609190.271153817686.0189eef162c828c942ae6c6b5664e6b1%22%2C%22team%22%3A%7B%22id%22%3A%22T7Z4HQ1L6%22%2C%22domain%22%3A%22blackbird-crew%22%7D%2C%22enterprise%22%3Anull%2C%22is_enterprise_install%22%3Afalse%2C%22channel%22%3A%7B%22id%22%3A%22D0341CNTLM8%22%2C%22name%22%3A%22directmessage%22%7D%2C%22message%22%3A%7B%22bot_id%22%3A%22B03417WRY11%22%2C%22type%22%3A%22message%22%2C%22text%22%3A%22Should+I+adjust+the+number+of+hours+for+System+2+work%3F+You+have+until+end+of+day.%22%2C%22user%22%3A%22U03417K2FR8%22%2C%22ts%22%3A%221645904837.581049%22%2C%22team%22%3A%22T7Z4HQ1L6%22%2C%22blocks%22%3A%5B%7B%22type%22%3A%22section%22%2C%22block_id%22%3A%22l7%5C%2F%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%22Should+I+adjust+the+number+of+hours+for+System+2+work%3F+You+have+until+end+of+day.%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22actions%22%2C%22block_id%22%3A%22M%5C%2FE%22%2C%22elements%22%3A%5B%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%228zN%3D%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%220%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22Q3gd8%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%222%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22EiU%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%224%22%2C%22emoji%22%3Afalse%7D%7D%2C%7B%22type%22%3A%22button%22%2C%22action_id%22%3A%22rtsA%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%226%22%2C%22emoji%22%3Afalse%7D%7D%5D%7D%5D%7D%2C%22state%22%3A%7B%22values%22%3A%7B%7D%7D%2C%22response_url%22%3A%22https%3A%5C%2F%5C%2Fhooks.slack.com%5C%2Factions%5C%2FT7Z4HQ1L6%5C%2F3163761419107%5C%2F8OI44EMzlWemCaoFurG2ch7m%22%2C%22actions%22%3A%5B%7B%22action_id%22%3A%22rtsA%22%2C%22block_id%22%3A%22M%5C%2FE%22%2C%22text%22%3A%7B%22type%22%3A%22plain_text%22%2C%22text%22%3A%226%22%2C%22emoji%22%3Afalse%7D%2C%22type%22%3A%22button%22%2C%22action_ts%22%3A%221645904928.633379%22%7D%5D%7D"#;
        parse_slack_payload(response).unwrap();
    }

    #[test]
    fn test_validate_signature() {
        let signing_secret = "8f742231b10e8888abcd99yyyzzz85a5";
        let body = "token=xyzz0WbapA4vBCDEFasx0q6G&team_id=T1DC2JH3J&team_domain=testteamnow&channel_id=G8PSS9T3V&channel_name=foobar&user_id=U2CERLKJA&user_name=roadrunner&command=%2Fwebhook-collect&text=&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FT1DC2JH3J%2F397700885554%2F96rGlfmibIGlgcZRskXaIFfN&trigger_id=398738663015.47445629121.803a0bc887a14d10d2c447fce8b6703c";
        let request_timestamp = "1531420618";
        let signature = "v0=a2114d57b48eac39b9ad189dd8316235a7b4a8d21a10bd27519666489c69b503";
        validate_signature(signing_secret, request_timestamp, signature, body).unwrap();
    }
}

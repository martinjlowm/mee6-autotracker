use ::lib::services::dynamodb::dynamodb;
use anyhow::Result;
use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::prelude::Utc;
use chrono::Duration;
use jemallocator::Jemalloc;
use lambda_runtime::handler_fn;
use lazy_static::lazy_static;
use reqwest::header::CONTENT_TYPE;
use serde_derive::{Deserialize, Serialize};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Deserialize, Serialize, Debug)]
struct Project {
    id: i64,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Task {
    id: i64,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct User {
    id: i64,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct TimeEntry {
    id: i64,
    spent_date: String,
    user: User,
    project: Project,
    task: Task,
    // user_assignment: UserAssignment,
    // task_assignment: TaskAssignment,
    hours: f64,
}

#[derive(Deserialize, Serialize, Debug)]
struct TimeEntriesResponse {
    time_entries: Vec<TimeEntry>,
}

#[derive(Deserialize, Serialize)]
struct Text {
    r#type: String,
    emoji: bool,
    text: String,
}

#[derive(Deserialize, Serialize)]
struct Element {
    r#type: String,
    text: Text,
}

#[derive(Deserialize, Serialize)]
struct Field {
    r#type: String,
    text: String,
}

#[derive(Deserialize, Serialize)]
struct Section {
    r#type: String,
    text: Text,
}

#[derive(Deserialize, Serialize)]
struct Block {
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elements: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<Field>>,
}

#[derive(Deserialize, Serialize)]
struct SlackQuestion {
    channel: String,
    // user: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // as_user: Option<bool>,
    text: String,
    blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Profile {
    display_name: String,
    display_name_normalized: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Member {
    id: String,
    profile: Profile,
}

#[derive(Deserialize, Serialize)]
struct UsersList {
    members: Vec<Member>,
}

lazy_static! {
    static ref SLACK: reqwest::Client = {
        let slack_token: String = std::env::var("SLACK_TOKEN").expect("SLACK_TOKEN is not set!");

        let slack_client_builder = reqwest::Client::builder();

        let mut slack_headers = http::header::HeaderMap::new();
        slack_headers.insert(
            "Authorization",
            http::header::HeaderValue::from_str(
                format!("Bearer {token}", token = &slack_token).as_str(),
            )
            .unwrap(),
        );

        slack_headers.insert(
            CONTENT_TYPE,
            http::header::HeaderValue::from_str("application/json; charset=utf-8").unwrap(),
        );

        slack_client_builder
            .user_agent("reqwest")
            .default_headers(slack_headers)
            .build()
            .unwrap()
    };
}

async fn handler(_event: CloudWatchEvent, _: lambda_runtime::Context) -> Result<()> {
    let response: UsersList = SLACK
        .get("https://slack.com/api/users.list")
        .send()
        .await?
        .json()
        .await?;

    let martinjlowm = response
        .members
        .into_iter()
        .find(|m| m.profile.display_name.eq_ignore_ascii_case("martinjlowm"))
        .unwrap();

    let msg = "Should I adjust the number of hours for System 2 work? You have until end of day.";

    let slack_question = SlackQuestion {
        channel: martinjlowm.id,
        text: msg.into(),
        blocks: vec![
            Block {
                r#type: "section".into(),
                text: Some(Text {
                    r#type: "plain_text".into(),
                    emoji: false,
                    text: msg.into(),
                }),
                fields: None,
                elements: None,
            },
            Block {
                r#type: "actions".into(),
                text: None,
                fields: None,
                elements: Some(
                    (0..8)
                        .step_by(2)
                        .map(|i| Element {
                            r#type: "button".into(),
                            text: Text {
                                r#type: "plain_text".into(),
                                emoji: false,
                                text: i.to_string(),
                            },
                        })
                        .collect(),
                ),
            },
        ],
    };

    SLACK
        .post("https://slack.com/api/chat.postMessage")
        .body(serde_json::to_string(&slack_question).unwrap())
        .send()
        .await?;

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
        .expression_attribute_names("#ttl", "ttl")
        .expression_attribute_values(
            ":ttl",
            AttributeValue::N(format!(
                "{}",
                now.timestamp() + Duration::hours(8).num_seconds()
            )),
        )
        .update_expression("SET #hours = :hours, #ttl = :ttl")
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

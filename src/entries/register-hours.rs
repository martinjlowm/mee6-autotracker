use std::collections::HashMap;

use anyhow::Result;
use aws_lambda_events::event::dynamodb::Event;
use lazy_static::lazy_static;
use lambda_runtime::handler_fn;
use jemallocator::Jemalloc;
use serde_derive::{Deserialize, Serialize};
use serde_dynamo::from_item;

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
struct TTLItem {
    ttl: u64,
    hours: u64,
}

lazy_static! {
    static ref HARVEST: reqwest::Client = {
        let account_id: String = std::env::var("HARVEST_ACCOUNT_ID").expect("HARVEST_ACCOUNT_ID is not set!");
        let harvest_token: String = std::env::var("HARVEST_TOKEN").expect("HARVEST_TOKEN is not set!");

        let client_builder = reqwest::Client::builder();

        let mut headers = http::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            http::header::HeaderValue::from_str(
                format!("Bearer {token}", token = &harvest_token).as_str(),
            )
                .unwrap(),
        );
        headers.insert(
            "Harvest-Account-ID",
            http::header::HeaderValue::from_str(format!("{id}", id = account_id).as_str()).unwrap(),
        );

        client_builder
            .user_agent("reqwest")
            .default_headers(headers)
            .build()
            .unwrap()
    };
}

async fn handler(event: Event, _: lambda_runtime::Context) -> Result<()> {
    dbg!(event.clone());

    let removed_items: Vec<HashMap<_,_>> = event.records.into_iter().filter_map(|record| {
        if !record.event_name.eq_ignore_ascii_case("REMOVE") {
            return None;
        }

        Some(record.change.old_image)
    }).collect();


    // let response = HARVEST
    //     .get("https://api.harvestapp.com/v2/time_entries")
    //     .send()
    //     .await?
    //     .json::<TimeEntriesResponse>()
    //     .await?;
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

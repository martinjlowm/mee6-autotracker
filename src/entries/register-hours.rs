use ::lib::types::harvest::{
    CreateEntryRequest, CreateEntryResponse, MeResponse, ProjectAssignment,
    ProjectAssignmentsResponse,
};
use anyhow::{Context, Result};
use aws_lambda_events::event::dynamodb::{attributes::AttributeValue, Event};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use futures::{future::join_all, stream::FuturesUnordered};
use jemallocator::Jemalloc;
use lambda_runtime::handler_fn;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Deserialize, Serialize)]
struct ActionItem {
    ttl: u64,
    hours: u64,
}

lazy_static! {
    static ref HARVEST: reqwest::Client = {
        let account_id: String =
            std::env::var("HARVEST_ACCOUNT_ID").expect("HARVEST_ACCOUNT_ID is not set!");
        let harvest_token: String =
            std::env::var("HARVEST_TOKEN").expect("HARVEST_TOKEN is not set!");

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

async fn register_hours(
    user_id: i64,
    project_assignments: Vec<ProjectAssignment>,
    timestamp: NaiveDateTime,
    hours: f64,
) -> Result<()> {
    let project_assignment = project_assignments
        .into_iter()
        .find(|assignment| {
            assignment
                .project
                .name
                .eq_ignore_ascii_case("System2 Development Hours")
        })
        .with_context(|| "Failed to find project")?;

    let task_assignment = project_assignment
        .task_assignments
        .into_iter()
        .find(|assignment| assignment.task.name.eq_ignore_ascii_case("Development"))
        .with_context(|| "Failed to find task")?;

    let create_entry = CreateEntryRequest {
        user_id: Some(user_id),
        project_id: project_assignment.project.id,
        task_id: task_assignment.task.id,
        spent_date: timestamp,
        hours: Some(hours),
        notes: None,
    };

    let response: CreateEntryResponse = HARVEST
        .post("https://api.harvestapp.com/v2/time_entries")
        .json(&create_entry)
        .send()
        .await?
        .json()
        .await?;

    log::info!("Created time entry w. {:?}", response);

    Ok(())
}

pub fn split_into_naive_datetime(field: &str) -> Option<NaiveDateTime> {
    let timestamp = field.split('|').nth(1)?.to_string();
    NaiveDate::parse_from_str(timestamp.as_str(), "%Y-%m-%d")
        .ok()
        .map(|date| date.and_time(NaiveTime::from_hms(0, 0, 0)))
}

pub async fn handler(event: Event, _: lambda_runtime::Context) -> Result<()> {
    let removed_items = event.records.into_iter().filter_map(|record| {
        if !record.event_name.eq_ignore_ascii_case("REMOVE") {
            return None;
        }

        let image = record.change.old_image;
        let timestamp = match image.get("pk").with_context(|| "Item had no pk field") {
            Ok(AttributeValue::String(value)) => Some(split_into_naive_datetime(value)?),
            _ => None,
        }?;

        let hours = match image
            .get("hours")
            .with_context(|| "Item had no hours field")
        {
            Ok(AttributeValue::Number(value)) => Some(value.clone()),
            _ => None,
        }?;

        Some((timestamp, hours))
    });

    let MeResponse { id: user_id, .. } = HARVEST
        .get("https://api.harvestapp.com/v2/users/me")
        .send()
        .await?
        .json()
        .await?;

    let ProjectAssignmentsResponse {
        project_assignments,
    } = HARVEST
        .get("https://api.harvestapp.com/v2/users/me/project_assignments")
        .send()
        .await?
        .json()
        .await?;

    let result = join_all(
        removed_items
            .map(move |(timestamp, hours)| {
                let assignments = project_assignments.clone();
                Box::pin(
                    async move { register_hours(user_id, assignments, timestamp, hours).await },
                )
            })
            .collect::<FuturesUnordered<_>>(),
    )
    .await;

    log::info!("Registered hours for {} entries", result.len());

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
    use super::{register_hours, split_into_naive_datetime, HARVEST};
    use ::lib::types::harvest::{MeResponse, ProjectAssignmentsResponse};

    #[tokio::test]
    async fn test_response_parsing() {
        dotenv::dotenv().ok();

        let MeResponse { id: user_id, .. } = HARVEST
            .get("https://api.harvestapp.com/v2/users/me")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let ProjectAssignmentsResponse {
            project_assignments,
        } = HARVEST
            .get("https://api.harvestapp.com/v2/users/me/project_assignments")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let timestamp = split_into_naive_datetime("timestamp|2022-02-27").unwrap();
        let hours = "2".parse::<f64>().ok().unwrap();

        match register_hours(user_id, project_assignments, timestamp, hours).await {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }
}

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Task {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TimeEntry {
    pub id: i64,
    pub spent_date: String,
    pub user: User,
    pub project: Project,
    pub task: Task,
    // user_assignment: UserAssignment,
    // task_assignment: TaskAssignment,
    pub hours: f64,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<TimeEntry>,
}

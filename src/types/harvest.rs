use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProjectAssignment {
    pub id: i64,
    pub project: Project,
    pub task_assignments: Vec<TaskAssignment>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TaskAssignment {
    pub id: i64,
    pub task: Task,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<TimeEntry>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MeResponse {
    pub id: i64,
    pub first_name: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProjectAssignmentsResponse {
    pub project_assignments: Vec<ProjectAssignment>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateEntryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    pub project_id: i64,
    pub task_id: i64,
    pub spent_date: NaiveDateTime,
    // We format this manually because
    // pub spent_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    // An object containing the id, group_id, account_id, and permalink of the
    // external reference. - whatever that means
    // pub external_reference: Option<object>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateEntryResponse {
    pub id: i64,
    pub spent_date: String,
    pub hours: f64,
    pub is_running: bool,
}

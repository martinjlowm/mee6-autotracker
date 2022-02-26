use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Text {
    pub r#type: String,
    pub emoji: bool,
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Element {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    pub text: Text,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Field {
    pub r#type: String,
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Section {
    pub r#type: String,
    pub text: Text,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Block {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Field>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SlackQuestion {
    pub channel: String,
    // user: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // as_user: Option<bool>,
    pub text: String,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Profile {
    pub display_name: String,
    pub display_name_normalized: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Member {
    pub id: String,
    pub profile: Profile,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UsersList {
    pub members: Vec<Member>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Container {
    pub r#type: String,
    pub message_ts: String,
    pub channel_id: String,
    pub is_ephemeral: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Team {
    pub id: String,
    pub domain: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Message {
    pub bot_id: String,
    pub r#type: String,
    pub text: String,
    pub user: String,
    pub ts: String,
    pub team: String,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct State {
    pub values: Value,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Action {
    pub action_id: String,
    pub block_id: String,
    pub text: Text,
    pub r#type: String,
    pub action_ts: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Response {
    pub r#type: String,
    pub user: User,
    pub api_app_id: String,
    pub token: String,
    pub container: Container,
    pub trigger_id: String,
    pub team: Team,
    pub enterprise: Option<String>,
    pub is_enterprise_install: bool,
    pub channel: Channel,
    pub message: Message,
    pub state: State,
    pub response_url: String,
    pub actions: Vec<Action>,
}

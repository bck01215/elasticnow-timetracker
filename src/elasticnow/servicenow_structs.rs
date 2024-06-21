use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SNResult<T = ServiceNowResultResponse> {
    pub result: T,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ServiceNowResultResponse {
    User(Vec<UserGroupResult>),
    SysId(SysIdResult),
    SysIds(Vec<SysIdResult>),
    CHG(CHGCreation),
}

#[derive(Deserialize, Debug)]
pub struct SysIdResult {
    pub sys_id: String,
    pub sys_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserGroupResult {
    #[serde(rename = "u_default_group")]
    pub default_group: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketCreation {
    #[serde(rename = "assignment_group")]
    pub assignment_group: String,
    #[serde(rename = "short_description")]
    pub short_description: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "cmdb_ci", skip_serializing_if = "Option::is_none")]
    pub configuration_item: Option<String>,
    #[serde(rename = "sys_class_name", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(rename = "priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(rename = "cat_item", skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    #[serde(rename = "u_sla_type", skip_serializing_if = "Option::is_none")]
    pub sla_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CHGCreation {
    pub sys_id: DisplayAndValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayAndValue {
    pub display_value: String,
    pub value: String,
}

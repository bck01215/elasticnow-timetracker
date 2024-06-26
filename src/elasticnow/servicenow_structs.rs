use crate::elasticnow::servicenow;
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
    TimeWorked(TimeWorked),
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
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayAndLink {
    pub display_value: String,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkAndValue {
    pub link: String,
    pub value: String,
}
impl LinkAndValue {
    pub async fn get_link<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        sn_client: &servicenow::ServiceNow,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let resp = sn_client.get(&self.link).await?;
        if !resp.status().is_success() {
            return Err(format!("HTTP Error while querying ServiceNow: {}", resp.status()).into());
        }
        let result = servicenow::debug_resp_json_deserialize::<SNResult<T>>(resp).await;
        if result.is_err() {
            let error_msg = format!("JSON error: {}", result.unwrap_err());
            tracing::error!("{}", error_msg);
            return Err(error_msg.into());
        }
        Ok(result.unwrap().result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimeWorkedTask {
    DisplayAndValue(DisplayAndValue),
    EmptyString(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeWorked {
    pub time_in_seconds: String,
    pub task: String,
    #[serde(rename = "u_category")]
    pub category: String,
}

impl TimeWorked {
    pub fn get_nice_name_category(&self) -> String {
        match self.category.as_str() {
            "certs_prodev_training" => "Training".to_string(),
            "clerical" => "Clerical".to_string(),
            "univ_events" => "University Events".to_string(),
            _ => self.category.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostCenter {
    pub cost_center: DisplayAndValue,
    pub task: DisplayAndValue,
}

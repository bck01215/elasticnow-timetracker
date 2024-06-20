use chrono::{TimeZone, Utc};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::debug;

#[derive(Deserialize)]
struct SysIdResponse {
    result: SysIdResult,
}

#[derive(Deserialize)]
struct SysIdResult {
    sys_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TicketCreation {
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

#[derive(Debug, Deserialize)]
struct UserResponse {
    pub result: Vec<UserResult>,
}

#[derive(Debug, Deserialize)]
struct UserResult {
    #[serde(rename = "u_default_group")]
    pub default_group: String,
}

pub struct ServiceNow {
    username: String,
    password: String,
    instance: String,
    pub client: Client,
}

impl ServiceNow {
    pub fn new(username: &str, password: &str, instance: &str) -> Self {
        let client = reqwest::Client::new();
        let set_instance = format!("https://{}.service-now.com", instance);
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
            instance: set_instance,
            client,
        }
    }
    async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        debug!("Getting {}", path);
        self.client
            .get(path)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
    }
    async fn post_json(
        &self,
        path: &str,
        json: serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        debug!("Getting {}", path);
        self.client
            .post(path)
            .json(&json)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
    }
    pub async fn get_user_group(&self, username: &str) -> Result<String, Box<dyn Error>> {
        let resp = self.get(&format!(
            "{}/api/now/table/sys_user?user_name={}&sysparm_limit=1&sysparm_display_value=true&sysparm_exclude_reference_link=true&sysparm_fields=u_default_group",
            self.instance, username
        ))
        .await?;

        if !resp.status().is_success() {
            return Err(format!("HTTP Error while querying ServiceNow: {}", resp.status()).into());
        }

        let user_response = resp.json::<UserResponse>().await?;
        let group = user_response.result.first().unwrap();

        Ok(group.default_group.clone())
    }
    pub async fn add_time_to_ticket(
        &self,
        ticket_id: &str,
        time_worked: &str,
        comment: &str,
    ) -> Result<(), Box<dyn Error>> {
        let time_worked = time_add_to_epoch(time_worked)?;
        let post_body = serde_json::json!({
            "time_worked": time_worked,
            "comments": comment,
            "task": ticket_id
        });
        self.post_json(
            &format!("{}/api/now/table/task_time_worked", self.instance),
            post_body,
        )
        .await?;
        Ok(())
    }
    // Returns the sys_id of created or errors
    pub async fn create_ticket(
        &self,
        assignment_group: &str,
        description: &str,
    ) -> Result<String, Box<dyn Error>> {
        let ticket = TicketCreation {
            assignment_group: assignment_group.to_owned(),
            short_description: description.to_owned(),
            description: description.to_owned(),
            type_: Some("sc_req_item".to_owned()),
            priority: Some("4".to_owned()),
            configuration_item: None,
            item: None,
            sla_type: Some("server_specific".to_owned()),
        };
        let json_payload = serde_json::to_value(ticket);

        if json_payload.is_err() {
            return Err(format!("JSON error: {}", json_payload.unwrap_err()).into());
        }

        let resp = self
            .post_json(
                &format!("{}/api/now/table/sc_req_item", self.instance),
                json_payload.unwrap(),
            )
            .await?
            .json::<SysIdResponse>()
            .await?;

        Ok(resp.result.sys_id)
    }
}

pub fn time_add_to_epoch(time: &str) -> Result<String, Box<dyn Error>> {
    let time_regex = Regex::new(r"^(?:(\d+)h)?(?:(\d+)m)?$").unwrap();
    let captures = time_regex
        .captures(time)
        .ok_or("Invalid time format. Please use optional hh and/or mm")?;

    let hours: u32 = captures
        .get(1)
        .map_or(Ok(0), |h| h.as_str().parse())
        .unwrap();
    let minutes: u32 = captures
        .get(2)
        .map_or(Ok(0), |m| m.as_str().parse())
        .unwrap();
    if hours > 23 || minutes > 59 {
        return Err(
            "Invalid time format. Values must be below 23 for hours, 60 for minutes".into(),
        );
    }
    let epoch_time = (hours * 3600 + minutes * 60) as i64;
    if epoch_time == 0 {
        return Err("Time worked must be greater than 0 minutes".into());
    }
    let formatted_time = Utc
        .timestamp_opt(epoch_time, 0)
        .unwrap()
        .format("%Y-%m-%d+%H:%M:%S")
        .to_string();
    Ok(formatted_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_time_add_to_epoch_basic() {
        assert_eq!(time_add_to_epoch("1h2m").unwrap(), "1970-01-01+01:02:00");
    }

    #[test]
    fn test_time_add_to_epoch_too_many_hours() {
        assert_eq!(
            time_add_to_epoch("24h0m").unwrap_err().to_string(),
            "Invalid time format. Values must be below 23 for hours, 60 for minutes"
        );
    }

    #[test]
    fn test_time_add_to_epoch_too_many_minutes() {
        assert_eq!(
            time_add_to_epoch("0h60m").unwrap_err().to_string(),
            "Invalid time format. Values must be below 23 for hours, 60 for minutes"
        );
    }
    #[test]
    fn test_time_only_hour() {
        assert_eq!(time_add_to_epoch("1h").unwrap(), "1970-01-01+01:00:00");
    }

    #[test]
    fn test_time_only_minute() {
        assert_eq!(time_add_to_epoch("1m").unwrap(), "1970-01-01+00:01:00");
    }

    #[test]
    fn test_time_no_time() {
        assert_eq!(
            time_add_to_epoch("0h").unwrap_err().to_string(),
            "Time worked must be greater than 0 minutes"
        );
    }
}

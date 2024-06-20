use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use tracing::debug;

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
}

use reqwest::Client;
use tracing::debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "_score")]
    pub score: f64,
    #[serde(rename = "_source")]
    pub source: Ticket,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub comments: String,
    pub work_notes: String,
    pub description: String,
    pub short_description: String,
    pub active: String,
    pub assigned_to: String,
    pub assignment_group: String,
    pub sys_created_on: String,
    pub id: String,
    pub self_link: String,
    pub priority: String,
    pub number: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl ChooseOptions<SearchResult> for SearchResult {
    fn get_debug_string(&self) -> String {
        format!("{}: {}", self.source.number, self.source.short_description)
    }
    fn get_number(&self) -> String {
        self.source.number.clone()
    }
    fn get_id(&self) -> String {
        self.source.id.clone()
    }
}

pub trait ChooseOptions<T> {
    fn get_debug_string(&self) -> String;
    fn get_id(&self) -> String;
    fn get_number(&self) -> String;
}

pub struct ElasticNow {
    instance: String,
    pub client: Client,
}

impl ElasticNow {
    pub fn new(id: &str, instance: &str) -> Self {
        let cookie = format!("id={}", id);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(&cookie).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self {
            instance: instance.to_owned(),
            client,
        }
    }
    async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        debug!("Getting {}", path);
        self.client
            .post(self.instance.to_owned() + path)
            .json(&body)
            .send()
            .await
    }

    pub async fn get_keyword_tickets(
        &self,
        keywords: &str,
        bin: &str,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let body = serde_json::json!(
            [
                {
                    "term": {
                        "assignment_group": bin
                    }
                },
                {
                    "term": {
                        "active": "true"
                    }
                }
            ]
        );
        let keywords = urlencoding::encode(keywords);
        let resp = self
            .post_json(&format!("/tickets/{}", keywords), body)
            .await?;

        if !resp.status().is_success() {
            return Err(format!("HTTP Error while querying ElasticNow: {}", resp.status()).into());
        }

        let search_results: Vec<SearchResult> = resp.json().await?;

        Ok(search_results)
    }
}

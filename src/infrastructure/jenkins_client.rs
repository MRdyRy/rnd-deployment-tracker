use crate::domain::models::{JenkinsBuild, JenkinsBuildDetails, JenkinsFolderResponse, JenkinsJobResponse};
use reqwest::{Client, Response};
use serde_json::{json, Error as JsonError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JenkinsClientError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    ParseError(#[from] JsonError),
}

#[derive(Debug)]
pub struct JenkinsClient {
    client: Client,
    base_url: String,
    api_key: String,
    username: String,
}

impl JenkinsClient {
    pub fn new(base_url: &str, username: &str, api_token: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_token.to_string(),
            username: username.to_string(),
        }
    }

    pub async fn get_builds(&self, service_name: &str) -> Result<Vec<JenkinsBuild>, JenkinsClientError> {
        let url = format!("{}job/{}/api/json", self.base_url, service_name);
        log::info!("url : {}", url);
        let response = self.client.get(&url)
            .basic_auth(&self.username, Some(&self.api_key))
            .send().await?;
        log::info!("status: {}", response.status());
        log::info!("response : {:?}", response);
        self.parse_response(response).await
    }

    // Add new method to fetch services
    pub async fn get_services(&self, job_parent: String) -> Result<Vec<String>, JenkinsClientError> {

        log::info!("in get seervices !");
        let url: String = match job_parent.is_empty() {
            true => {format!("{}api/json", self.base_url)}
            false => {format!("{}{}api/json", self.base_url, job_parent)}
        };
        log::info!("url : {}", url);
        let response = self.client.get(&url)
            .basic_auth(&self.username, Some(&self.api_key))
            .send().await?;
        log::info!("status: {}", response.status());
        let folder_response: JenkinsFolderResponse = response.json().await?;

        // Filter only the jobs we want (example: filter by class)
        let services = folder_response.jobs
            .into_iter()
            .filter(|job| job.class == "hudson.model.FreeStyleProject") // adjust based on your job type
            .map(|job| job.name)
            .collect();

        Ok(services)
    }

    pub async fn get_build_details(
        &self,
        job_name: &str,
        build_number: i64
    ) -> Result<JenkinsBuildDetails, JenkinsClientError> {
        let url = format!(
            "{}{}/{}/api/json",
            self.base_url, job_name, build_number
        );
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    pub fn extract_committer(actions: &[serde_json::Value]) -> Option<String> {
        actions.iter()
            .find(|a| a.get("_class") == Some(&json!("hudson.model.CauseAction")))
            .and_then(|a| a.get("causes"))
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("userId"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
    }

    async fn parse_response(&self, response: Response) -> Result<Vec<JenkinsBuild>, JenkinsClientError> {
        let job_response: JenkinsJobResponse = response.json().await?;
        Ok(job_response.builds)
    }
}
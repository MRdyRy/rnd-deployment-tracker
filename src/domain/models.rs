use serde::{Deserialize, Serialize};

// Model for Jenkins API response
#[derive(Debug, Deserialize)]
pub struct JenkinsJobResponse {
    pub builds: Vec<JenkinsBuild>,
}

// Individual build information
#[derive(Debug, Deserialize)]
pub struct JenkinsBuild {
    pub number: i64,
    pub result: Option<String>,
    // pub timestamp: Option<i64>,
}

// Output model for our API
#[derive(Debug, Serialize)]
pub struct DeploymentSummary {
    pub total_deployments: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub avg_deployments_per_day: f64,
}

#[derive(Debug, Deserialize)]
pub struct JenkinsFolderResponse {
    pub jobs: Vec<JenkinsJob>,
}

#[derive(Debug, Deserialize)]
pub struct JenkinsJob {
    pub name: String,
    pub url: String,
    #[serde(rename = "_class")]
    pub class: String,
}

#[derive(Debug, Deserialize)]
pub struct JenkinsBuildDetails {
    pub actions: Vec<serde_json::Value>,
    pub result: Option<String>,
    pub duration: i64,
    pub timestamp: i64,
    pub full_display_name: String,
}

#[derive(Debug, Serialize)]
pub struct Activity {
    pub job_name: String,
    pub committer: Option<String>,
    pub status: Option<String>,
    pub duration_seconds: f64,
    pub timestamp: i64,
}
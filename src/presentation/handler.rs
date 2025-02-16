use axum::{Json, Extension};
use crate::application::deployment_service::DeploymentService;
use crate::domain::models::{Activity, DeploymentSummary};
use std::sync::Arc;
use axum::extract::Path;
use log::trace;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    JenkinsError(#[from] crate::infrastructure::jenkins_client::JenkinsClientError),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::JenkinsError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub async fn deployment_handler(
    Extension(service): Extension<Arc<DeploymentService>>,
) -> Result<Json<DeploymentSummary>, Error> {
    log::info!("IN CONTROLLER");
    let summary = service.get_summary().await?;
    Ok(Json(summary))
}

#[derive(serde::Deserialize)]
pub struct ActivityParams {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

pub async fn latest_activities_handler(
    Extension(service): Extension<Arc<DeploymentService>>,
    axum::extract::Query(params): axum::extract::Query<ActivityParams>,
) -> Result<Json<Vec<Activity>>, Error> {
    log::info!("IN CONTROLLER latest_activities_handler");
    let activities = service.get_latest_activities(params.limit).await?;
    Ok(Json(activities))
}

pub async fn list_services_handler(
    Extension(service): Extension<Arc<DeploymentService>>,
    Path(job_parent) : Path<String>,
) -> Result<Json<Vec<String>>, Error> {

    let services = service.get_list_services(&*job_parent).await?;
    Ok(Json(services))

}
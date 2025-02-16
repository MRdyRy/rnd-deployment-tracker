use axum::{Json, Extension};
use crate::application::deployment_service::DeploymentService;
use crate::domain::models::DeploymentSummary;
use std::sync::Arc;
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
mod domain{
    pub mod models;
}
mod infrastructure {
    pub mod jenkins_client;
}
mod application{
    pub mod deployment_service;
}
mod presentation {
    pub mod handler;
}

use application::deployment_service::DeploymentService;
use axum::{Extension, Router};
use infrastructure::jenkins_client::JenkinsClient;
use presentation::handler::deployment_handler;
use std::sync::Arc;
use crate::presentation::handler::{latest_activities_handler, list_services_handler};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let jenkins_client = JenkinsClient::new(
        "http://localhost:8080/job/ryan_labs/job/dev/",
        "rudy_ryanto",
        "112e669bb03ad2bb2bcbb7059c544b6793"
    );

    // 2. Fetch services dynamically
    let service_names = jenkins_client.get_services("".to_string())
        .await
        .expect("Failed to fetch services from Jenkins");

    if service_names.is_empty() {
        panic!("No services found in Jenkins");
    }

    let deployment_service = Arc::new(DeploymentService::new(
        jenkins_client,
        service_names
    ));

    let app = Router::new()
        .route("/api/deployments/summary", axum::routing::get(deployment_handler))
        .route("/api/services", axum::routing::get(list_services_handler))
        .route("/api/services/:job_parent", axum::routing::get(list_services_handler))
        .route("/api/activities/latest", axum::routing::get(latest_activities_handler))
        .layer(Extension(deployment_service));

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(addr, app).await.unwrap();
}
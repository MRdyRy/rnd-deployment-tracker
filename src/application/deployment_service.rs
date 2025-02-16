use crate::domain::models::{DeploymentSummary, JenkinsBuild};
use crate::infrastructure::jenkins_client::{JenkinsClient, JenkinsClientError};
use chrono::{DateTime, Utc};
use futures::future::try_join_all;

#[derive(Debug)]
pub struct DeploymentService {
    jenkins_client: JenkinsClient,
    service_names: Vec<String>,
}

impl DeploymentService {
    pub fn new(jenkins_client: JenkinsClient, service_names: Vec<String>) -> Self {
        Self {
            jenkins_client,
            service_names,
        }
    }

    pub async fn get_summary(&self) -> Result<DeploymentSummary, JenkinsClientError> {
        log::info!("in service");
        let futures = self.service_names
            .iter()
            .map(|service| self.jenkins_client.get_builds(service));

        let results = try_join_all(futures).await?;
        log::info!("results {}",results.len());
        let all_builds: Vec<_> = results.into_iter().flatten().collect();
        log::info!("all builds {}",all_builds.len());

        Ok(self.calculate_summary(&all_builds))
    }

    fn calculate_summary(&self, builds: &[JenkinsBuild]) -> DeploymentSummary {
        let total_deployments = builds.len();
        let (success_count, failure_count) = builds.iter().fold((0, 0), |(success, failure), build| {
            match build.result.as_deref() {
                Some("SUCCESS") => (success + 1, failure),
                Some("FAILURE") => (success, failure + 1),
                _ => (success, failure),
            }
        });

        // let avg_deployments = if total_deployments > 0 {
        //     let timestamps: Vec<DateTime<Utc>> = builds
        //         .iter()
        //         .filter_map(|b| DateTime::from_timestamp_millis(b.timestamp))
        //         .collect();
        // 
        //     let min_time = timestamps.iter().min().unwrap();
        //     let max_time = timestamps.iter().max().unwrap();
        //     let duration = max_time.signed_duration_since(*min_time);
        //     let days = duration.num_days().max(1) as f64;
        // 
        //     total_deployments as f64 / days
        // } else {
        //     0.0
        // };

        DeploymentSummary {
            total_deployments,
            success_count,
            failure_count,
            // avg_deployments_per_day: avg_deployments,
            avg_deployments_per_day: 0.0,
        }
    }
}
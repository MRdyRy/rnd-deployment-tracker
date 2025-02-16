use crate::domain::models::{Activity, DeploymentSummary, JenkinsBuild};
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

    pub async fn get_latest_activities(&self, limit: usize) -> Result<Vec<Activity>, JenkinsClientError> {
        let mut all_activities = Vec::new();

        // Get last build for each service
        for service in &self.service_names {
            log::info!("service name {}",service);
            if let Some(last_build) = self.jenkins_client.get_builds(service).await?.first() {
                let details = self.jenkins_client.get_build_details(service, last_build.number).await?;
                log::info!("details {}",details);

                all_activities.push(Activity {
                    job_name: details.full_display_name,
                    committer: JenkinsClient::extract_committer(&details.actions),
                    status: details.result,
                    duration_seconds: details.duration as f64 / 1000.0,
                    timestamp: details.timestamp,
                });
            }
        }

        // Sort by timestamp descending
        all_activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Take requested limit
        Ok(all_activities.into_iter().take(limit).collect())
    }

    pub async fn get_list_services(&self, job_parent: &str) -> Result<Vec<String>, JenkinsClientError> {
        let list_services = self.jenkins_client.get_services(job_parent.to_string()).await?;
        Ok(list_services)
    }
}
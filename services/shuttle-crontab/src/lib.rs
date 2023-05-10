use chrono::Utc;
use std::future::Future;
use tokio::time::sleep;

use cron::Schedule;
use shuttle_runtime::{async_trait, Service};

pub struct CronJob<F> {
    pub schedule: Schedule,
    pub job: fn() -> F,
}
pub struct CronService<F> {
    pub jobs: Vec<CronJob<F>>,
    // pub schedule: Schedule,
    // pub job: fn() -> F,
}

#[async_trait]
impl<F: Future<Output = ()> + Send + Sync + 'static> Service for CronService<F> {
    async fn bind(
        mut self,
        _addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_service::error::Error> {
        for job in self.jobs {
            tokio::spawn(async move {
                while let Some(next_run) = job.schedule.upcoming(Utc).next() {
                    let next_run_in = next_run
                        .signed_duration_since(chrono::offset::Utc::now())
                        .to_std()
                        .unwrap();
                    sleep(next_run_in).await;
                    (job.job)().await;
                }
            });
        }
        // TODO: some kind of infinite loop?
        sleep(std::time::Duration::from_secs(100)).await;

        Ok(())
    }
}

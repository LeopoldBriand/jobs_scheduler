use chrono::Utc;
use shlex::Shlex;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::process::Command;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use utils::parse_jobs;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    let jobs_content =
        fs::read_to_string("./jobs").expect("Should have been able to read the jobs file");
    let jobs = parse_jobs(jobs_content);
    let scheduler = JobScheduler::new().await?;
    for (name, cron, command) in jobs {
        scheduler
            .add(Job::new(cron.as_str(), move |_uuid, _l| {
                let timestamp = Utc::now().timestamp();
                let mut lex = Shlex::new(&command);
                let mut args = lex.by_ref().collect::<Vec<String>>();
                let output = Command::new(args.remove(0))
                    .args(args)
                    .output()
                    .unwrap_or_else(|_| panic!("Failed to execute process {}", name));
                if output.status.success() {
                    add_to_history(
                        name.clone(),
                        timestamp,
                        "SUCCESS",
                        std::str::from_utf8(&output.stderr).unwrap(),
                    )
                } else {
                    add_to_history(
                        name.clone(),
                        timestamp,
                        "ERROR",
                        std::str::from_utf8(&output.stderr).unwrap(),
                    )
                }
            })?)
            .await?;
    }
    scheduler.start().await?;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn add_to_history(name: String, timestamp: i64, status: &str, error_message: &str) {
    let line = name + "," + &timestamp.to_string() + "," + status + "," + error_message;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./history")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", line) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

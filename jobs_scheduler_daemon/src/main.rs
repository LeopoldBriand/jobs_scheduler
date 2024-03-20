use chrono::Utc;
use core::fmt;
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
                match Command::new(args.remove(0)).args(args).output() {
                    Ok(output) => {
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
                        add_to_log(LogType::DEBUG, format!("Process {} executed", name))
                    }
                    Err(_) => add_to_log(
                        LogType::ERROR,
                        format!("Failed to execute process {}", name),
                    ),
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
    match OpenOptions::new()
        .write(true)
        .append(true)
        .open("./history")
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", line) {
                add_to_log(LogType::ERROR, format!("Couldn't write to file: {}", e));
            }
        }
        Err(_) => add_to_log(LogType::ERROR, format!("Couldn't open file:")),
    }
}

enum LogType {
    DEBUG,
    ERROR,
}

impl fmt::Display for LogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogType::DEBUG => write!(f, "DEBUG"),
            LogType::ERROR => write!(f, "ERROR"),
        }
    }
}

fn add_to_log(log_type: LogType, text: String) {
    // TODO: write to log file
    println!("{}: {}", log_type, text);
}

use chrono::Utc;
use core::fmt;
use directories::UserDirs;
use shlex::Shlex;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use utils::parse_jobs;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    let js_dir = UserDirs::new().unwrap().home_dir().join("job_scheduler");
    let log_file = js_dir.join("logs");
    let history_file = js_dir.join("history");
    let jobs_file = js_dir.join("jobs");
    // Check if files exists else create
    if !js_dir.join("logs").exists() {
        fs::create_dir_all(js_dir).expect("Not allowed to create ~/job_scheduler folder");
        std::fs::File::create(log_file).expect("Not allowed to create ~/job_scheduler/logs file");
    } else {
        std::fs::File::create(log_file).expect("Not allowed to empty ~/job_scheduler/logs file");
    }
    if !history_file.exists() {
        std::fs::File::create(history_file.clone())
            .expect("Not allowed to create ~/job_scheduler/history file");
    }
    let jobs = match jobs_file.exists() {
        true => match fs::read_to_string(jobs_file) {
            Ok(jobs_content) => parse_jobs(jobs_content),
            Err(_) => {
                add_to_log(
                    LogType::ERROR,
                    format!("Should have been able to read the jobs file"),
                );
                vec![]
            }
        },
        false => {
            add_to_log(
                LogType::DEBUG,
                format!("No job file found in ~/job_scheduler/"),
            );
            vec![]
        }
    };

    let scheduler = JobScheduler::new().await?;
    for (name, cron, command) in jobs {
        scheduler
            .add(Job::new(cron.as_str(), move |_uuid, _l| {
                let history_file = UserDirs::new()
                    .unwrap()
                    .home_dir()
                    .join("job_scheduler")
                    .join("history");
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
                                &history_file,
                            )
                        } else {
                            add_to_history(
                                name.clone(),
                                timestamp,
                                "ERROR",
                                std::str::from_utf8(&output.stderr).unwrap(),
                                &history_file,
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

fn add_to_history(
    name: String,
    timestamp: i64,
    status: &str,
    error_message: &str,
    history_file: &Path,
) {
    let line = name + "," + &timestamp.to_string() + "," + status + "," + error_message;
    match OpenOptions::new()
        .write(true)
        .append(true)
        .open(history_file)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", line) {
                add_to_log(
                    LogType::ERROR,
                    format!("Couldn't write to history file: {}", e),
                );
            }
        }
        Err(_) => add_to_log(LogType::ERROR, format!("Couldn't open history file:")),
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
    let log_file = UserDirs::new()
        .unwrap()
        .home_dir()
        .join("job_scheduler")
        .join("logs");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(log_file)
        .unwrap();
    if let Err(e) = writeln!(file, "{}: {}", log_type, text) {
        println!("Error while writing to log file: {}", e);
    }
    println!("{}: {}", log_type, text);
}

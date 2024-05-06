use chrono::{DateTime, Utc};
use core::fmt;
use directories::UserDirs;
use std::fs::{self, OpenOptions};
use std::io::{self, prelude::*};
use std::path::Path;
use std::process::Command;
use std::thread;
use utils::parse_jobs;

fn main() {
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
    let mut jobs = match jobs_file.exists() {
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

    jobs.sort_by_key(|j| j.next_run);

    loop {
        let time_to_wait = Utc::now() - jobs[0].next_run;
        thread::sleep(time_to_wait.to_std().unwrap());
        match Command::new(jobs[0].command.clone()).spawn() {
            Ok(child) => {
                if let Some(child_stderr) = child.stderr {
                    let mut stderr_reader = io::BufReader::new(child_stderr);
                    let mut error_message = String::new();
                    stderr_reader.read_to_string(&mut error_message).unwrap();

                    add_to_history(
                        jobs[0].name.clone(),
                        Utc::now(),
                        "Error",
                        &error_message,
                        &history_file,
                    );
                } else {
                    add_to_history(jobs[0].name.clone(), Utc::now(), "Ok", "", &history_file);
                }
            }
            Err(err) => add_to_log(LogType::ERROR, err.to_string()),
        }
        jobs[0].get_next_run();
        jobs.sort_by_key(|j| j.next_run);
    }
}

fn add_to_history(
    name: String,
    timestamp: DateTime<Utc>,
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

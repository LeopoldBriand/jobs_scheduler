use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;

mod parser;

// const JOB_REGEX: &'static str = r"^([a-zA-Z_-]*)[[:blank:]]?:[[:blank:]]?(@(?:annually|yearly|monthly|weekly|daily|hourly|reboot)|(?:@every (?:\d+(?:ns|us|µs|ms|s|m|h))+)|(?:(?:(?:(?:\d+,)+\d+|(?:\d+(?:\\/|-)\d+)|\d+|\*) ?){5,7}))[[:blank:]](.*)$";
const JOB_REGEX: &str = concat!(
    r"^([a-zA-Z_-]*)[[:blank:]]*:[[:blank:]]*(", // name
    r"(?:(?:(?:\d+,)+\d+|(?:(?:\d+|\*)(?:\/|-)(?:\d+|\*))|\d+|\*) ?){5}", // cron
    ")[[:blank:]]+(.+)$"                         // command
);

pub type History = Vec<HistoryStatement>;

#[derive(Debug)]
pub struct HistoryStatement {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub name: String,
    pub cron: String,
    pub command: String,
    pub next_run: Option<DateTime<Utc>>,
}

impl Job {
    pub fn new(name: String, cron: String, command: String) -> Job {
        Job {
            name,
            cron,
            command,
            next_run: None,
        }
    }
}

pub fn parse_history(file_content: String) -> History {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file_content.as_bytes());
    let mut history: History = vec![];
    for record in reader.records().flatten() {
        history.push(HistoryStatement {
            name: record[0].to_owned(),
            timestamp: Utc
                .timestamp_millis_opt(record[1].parse::<i64>().unwrap())
                .unwrap(),
            status: record[2].to_owned(),
            error_message: record[3].to_owned(),
        })
    }
    history
}

pub fn parse_jobs(file_content: String) -> Vec<Job> {
    let regex = Regex::new(JOB_REGEX).unwrap();
    let mut jobs = vec![];
    for line in file_content.lines() {
        for (_, [name, cron, command]) in regex.captures_iter(line).map(|c| c.extract()) {
            jobs.push(Job::new(
                name.to_owned(),
                cron.to_owned(),
                command.to_owned(),
            ));
        }
    }
    jobs
}
pub fn parse_job(content: String) -> Option<Job> {
    let regex = Regex::new(JOB_REGEX).unwrap();
    regex.captures(&content).map(|caps| {
        Job::new(
            caps[1].to_string(),
            caps[2].to_string(),
            caps[3].to_string(),
        )
    })
}

pub fn get_next_run(cron: &str) -> DateTime<Utc> {
    parser::parse(cron);
    todo!();
}

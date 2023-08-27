use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;

pub type History = Vec<HistoryStatement>;

#[derive(Debug)]
pub struct HistoryStatement {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub error_message: String,
}

pub type Job = (String, String, String);

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
    let regex= Regex::new(r"^([a-zA-Z_-]*)[[:blank:]]?:[[:blank:]]?(@(?:annually|yearly|monthly|weekly|daily|hourly|reboot)|(?:@every (?:\d+(?:ns|us|µs|ms|s|m|h))+)|(?:(?:(?:(?:\d+,)+\d+|(?:\d+(?:\\/|-)\d+)|\d+|\*) ?){5,7}))[[:blank:]](.*)$").unwrap();
    let mut jobs = vec![];
    for line in file_content.lines() {
        for (_, [name, cron, command]) in regex.captures_iter(line).map(|c| c.extract()) {
            jobs.push((name.to_owned(), cron.to_owned(), command.to_owned()));
        }
    }
    jobs
}
pub fn parse_job(content: String) -> Option<Job> {
    let regex= Regex::new(r"^([a-zA-Z_-]*)[[:blank:]]?:[[:blank:]]?(@(?:annually|yearly|monthly|weekly|daily|hourly|reboot)|(?:@every (?:\d+(?:ns|us|µs|ms|s|m|h))+)|(?:(?:(?:(?:\d+,)+\d+|(?:\d+(?:\\/|-)\d+)|\d+|\*) ?){5,7}))[[:blank:]](.*)$").unwrap();
    regex.captures(&content).map(|caps| {
        (
            caps[1].to_string(),
            caps[2].to_string(),
            caps[3].to_string(),
        )
    })
}
#[cfg(test)]
mod tests {}

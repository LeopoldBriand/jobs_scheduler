use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use regex::Regex;

mod parser;

// const JOB_REGEX: &'static str = r"^([a-zA-Z_-]*)[[:blank:]]?:[[:blank:]]?(@(?:annually|yearly|monthly|weekly|daily|hourly|reboot)|(?:@every (?:\d+(?:ns|us|µs|ms|s|m|h))+)|(?:(?:(?:(?:\d+,)+\d+|(?:\d+(?:\\/|-)\d+)|\d+|\*) ?){5,7}))[[:blank:]](.*)$";
const JOB_REGEX: &str = concat!(
    r"^([a-zA-Z_-]*)[[:blank:]]*:[[:blank:]]*(", // name
    r"(?:(?:(?:\d+,)+\d+|(?:(?:\d+|\*)(?:\/|-)(?:\d+|\*))|\d+|\*) ?){5}", // cron
    ")[[:blank:]]+(.+)$"                         // command
);

const LOOP_LIMIT: u32 = 128;

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
    pub next_run: DateTime<Utc>,
}

impl Job {
    pub fn new(name: String, cron: String, command: String) -> Job {
        Job {
            name,
            next_run: get_next_run(&cron),
            cron,
            command,
        }
    }
}

impl Job {
    pub fn get_next_run(&mut self) {
        self.next_run = get_next_run(&self.cron)
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
    get_next_run_from(Utc::now(), cron)
}

pub fn get_next_run_from(start: DateTime<Utc>, cron: &str) -> DateTime<Utc> {
    let fields = parser::parse(cron);
    let mut current_date = start.clone();
    let mut step_count: u32 = 0;
    let days_in_month = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    while step_count < LOOP_LIMIT {
        step_count += 1;

        // Match day of month and day of week
        let is_match_day_of_month = match_schedule(current_date.day(), &fields[2]);
        let is_match_day_of_week =
            match_schedule((current_date.weekday() as u8) as u32, &fields[4]);
        let is_day_of_month_wildcard =
            fields[2].len() >= days_in_month[current_date.month() as usize];
        let is_day_of_week_wildcard = fields[4].len() == 7;
        if (!is_match_day_of_month && (!is_match_day_of_week || is_day_of_week_wildcard))
            || (!is_day_of_month_wildcard && is_day_of_week_wildcard && !is_match_day_of_month)
            || (is_day_of_month_wildcard && !is_day_of_week_wildcard && !is_match_day_of_week)
        {
            current_date += Duration::days(1);
            continue;
        }

        // Match month
        if !match_schedule(current_date.month(), &fields[3]) {
            let (year, month) = (current_date.year(), current_date.month());
            let (next_year, next_month) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            current_date = Utc
                .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
                .unwrap();
            continue;
        }

        // Match hour
        if !match_schedule(current_date.hour(), &fields[1]) {
            current_date += Duration::hours(1);
            continue;
        }

        // Match minute
        if !match_schedule(current_date.minute(), &fields[0]) {
            current_date += Duration::minutes(1);
            continue;
        }
        break;
    }
    if step_count > LOOP_LIMIT {
        todo!("error")
    }
    return current_date;
}

fn match_schedule(value: u32, seq: &Vec<u32>) -> bool {
    for el in seq {
        if el >= &value {
            return el == &value;
        }
    }
    return seq[0] == value;
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::get_next_run_from;

    #[test]
    fn simple_cron() {
        let start = Utc.with_ymd_and_hms(2024, 05, 1, 0, 0, 0).unwrap();
        let date = get_next_run_from(start, "5 4 * * 6");
        assert_eq!(date.to_string(), "2024-05-05 04:05:00 UTC");
    }
}

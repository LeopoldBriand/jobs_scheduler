use directories::UserDirs;
use std::{error::Error, fs};
use utils::{parse_history, parse_jobs};

mod actions;
mod app;
mod events;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let js_dir = UserDirs::new().unwrap().home_dir().join("job_scheduler");
    let history_file = js_dir.join("history");
    let jobs_file = js_dir.join("jobs");

    // get saved jobs
    let jobs = match fs::read_to_string(jobs_file) {
        Ok(jobs_content) => parse_jobs(jobs_content),
        Err(_) => {
            println!("Unable to read the jobs file");
            vec![]
        }
    };

    // get jobs history
    let history_content =
        fs::read_to_string(history_file).expect("Should have been able to read the history file");
    let history = parse_history(history_content);

    app::run(history, jobs)
}

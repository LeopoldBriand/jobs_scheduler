use std::{error::Error, fs};
use utils::{parse_history, parse_jobs};

mod actions;
mod app;
mod events;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // get saved jobs
    let jobs_content =
        fs::read_to_string("./jobs").expect("Should have been able to read the jobs file");
    let jobs = parse_jobs(jobs_content);

    // get jobs history
    let history_content =
        fs::read_to_string("./history").expect("Should have been able to read the history file");
    let history = parse_history(history_content);

    app::run(history, jobs)
}

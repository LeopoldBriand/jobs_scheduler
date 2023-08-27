use crate::{actions::JobList, events::event_loop};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use std::{error::Error, io};
use utils::{History, Job};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Error,
}
pub struct Input {
    pub input: String,
    pub cursor_position: usize,
    pub input_mode: InputMode,
}

#[derive(PartialEq)]
pub enum InputSwitch {
    NameInput,
    CronInput,
}

#[derive(PartialEq)]
pub enum State {
    NotEditing,
    EditingJob,
    AddingJob(InputSwitch),
    DeletingJob,
}
pub struct App {
    pub jobs: JobList<Job>,
    pub history: History,
    pub name_input: Input,
    pub cron_input: Input,
    pub current_state: State,
}

impl App {
    fn new(history: History, jobs: Vec<Job>) -> App {
        App {
            jobs: JobList::with_items(jobs),
            history,
            name_input: Input {
                input: String::new(),
                cursor_position: 0,
                input_mode: InputMode::Normal,
            },
            cron_input: Input {
                input: String::new(),
                cursor_position: 0,
                input_mode: InputMode::Normal,
            },
            current_state: State::NotEditing,
        }
    }
    pub fn get_selected_job(&self) -> Option<(String, String, String)> {
        self.jobs
            .state
            .selected()
            .map(|index| self.jobs.items[index].clone())
    }
    pub fn get_selected_job_as_strings(&self) -> (String, String) {
        match self.get_selected_job() {
            Some(item) => (item.0.to_owned(), item.1 + " " + &item.2),
            None => (String::new(), String::new()),
        }
    }
}

pub fn run(history: History, jobs: Vec<Job>) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(history, jobs);
    let res = event_loop(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

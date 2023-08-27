use std::fs;

use ratatui::widgets::ListState;
use utils::parse_job;

use crate::app::{App, Input};

pub struct JobList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> JobList<T> {
    pub fn with_items(items: Vec<T>) -> JobList<T> {
        let mut list = JobList {
            state: ListState::default(),
            items,
        };
        list.next();
        list
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Input {
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    pub fn backspace_char(&mut self) {
        if self.cursor_position != 0 {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
    pub fn delete_char(&mut self) {
        if self.cursor_position != self.input.len() - 1 {
            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(self.cursor_position);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(self.cursor_position + 1);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}

impl App {
    pub fn append_job(&mut self) -> Option<()> {
        let job_input: String =
            self.name_input.input.clone() + ": " + self.cron_input.input.as_str();
        match parse_job(job_input) {
            Some(new_job) => {
                self.jobs.items.push(new_job);
                self.jobs.state.select(Some(self.jobs.items.len() - 1));
                self.write_jobs();
                Some(())
            }
            None => None,
        }
    }
    pub fn modify_job(&mut self) -> Option<()> {
        let job_input: String =
            self.name_input.input.clone() + ": " + self.cron_input.input.as_str();
        match parse_job(job_input) {
            Some(modified_job) => {
                let index = self.jobs.state.selected().unwrap();
                self.jobs.items[index] = modified_job;
                self.write_jobs();
                Some(())
            }
            None => None,
        }
    }
    pub fn delete_job(&mut self) {
        if let Some(index) = self.jobs.state.selected() {
            self.jobs.items.remove(index);
            if index > 0 {
                self.jobs.state.select(Some(index - 1));
            } else {
                self.jobs.state.select(None);
            }
            self.write_jobs();
        }
    }
    fn write_jobs(&self) {
        let content: Vec<String> = self
            .jobs
            .items
            .iter()
            .map(|el| format!("{}: {} {}", el.0, el.1, el.2))
            .collect();
        fs::write("./jobs", content.join("\n")).unwrap();
    }
}

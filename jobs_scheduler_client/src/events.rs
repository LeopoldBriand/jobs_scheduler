use crate::{
    app::{App, Input, InputMode, InputSwitch, State},
    ui::draw,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, Terminal};
use std::io;

pub fn event_loop<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.current_state {
                    State::NotEditing => match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Down => app.jobs.next(),
                        KeyCode::Up => app.jobs.previous(),
                        KeyCode::Enter => {
                            app.cron_input.input_mode = InputMode::Editing;
                            app.current_state = State::EditingJob;
                        }
                        KeyCode::Char('a') => {
                            app.name_input.input_mode = InputMode::Editing;
                            app.cron_input.input_mode = InputMode::Editing;
                            app.current_state = State::AddingJob(InputSwitch::NameInput);
                            app.name_input.input = String::new();
                            app.cron_input.input = String::new();
                        }
                        KeyCode::Char('d') => app.current_state = State::DeletingJob,
                        _ => {}
                    },
                    State::EditingJob => match key.code {
                        KeyCode::Enter => match app.modify_job() {
                            Some(_) => {
                                app.cron_input.input_mode = InputMode::Normal;
                                app.cron_input.reset_cursor();
                                app.current_state = State::NotEditing;
                            }
                            None => {
                                app.cron_input.input_mode = InputMode::Error;
                            }
                        },
                        KeyCode::Char(to_insert) => app.cron_input.enter_char(to_insert),
                        KeyCode::Backspace => app.cron_input.backspace_char(),
                        KeyCode::Delete => app.cron_input.delete_char(),
                        KeyCode::Left => app.cron_input.move_cursor_left(),
                        KeyCode::Right => app.cron_input.move_cursor_right(),
                        KeyCode::Esc => {
                            app.cron_input.input_mode = InputMode::Normal;
                            app.cron_input.reset_cursor();
                            app.current_state = State::NotEditing;
                        }
                        _ => {}
                    },
                    State::AddingJob(ref input_switch_value) => {
                        let selected_input: &mut Input = match input_switch_value {
                            InputSwitch::NameInput => &mut app.name_input,
                            InputSwitch::CronInput => &mut app.cron_input,
                        };
                        match key.code {
                            KeyCode::Enter => match app.append_job() {
                                Some(_) => {
                                    app.name_input.input_mode = InputMode::Normal;
                                    app.cron_input.input_mode = InputMode::Normal;
                                    app.name_input.reset_cursor();
                                    app.cron_input.reset_cursor();
                                    app.current_state = State::NotEditing;
                                }
                                None => {
                                    app.name_input.input_mode = InputMode::Error;
                                    app.cron_input.input_mode = InputMode::Error;
                                }
                            },
                            KeyCode::Char(to_insert) => selected_input.enter_char(to_insert),
                            KeyCode::Backspace => selected_input.backspace_char(),
                            KeyCode::Delete => selected_input.delete_char(),
                            KeyCode::Left => selected_input.move_cursor_left(),
                            KeyCode::Right => selected_input.move_cursor_right(),
                            KeyCode::Tab => {
                                app.current_state = match input_switch_value {
                                    InputSwitch::NameInput => {
                                        State::AddingJob(InputSwitch::CronInput)
                                    }
                                    InputSwitch::CronInput => {
                                        State::AddingJob(InputSwitch::NameInput)
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.name_input.input_mode = InputMode::Normal;
                                app.cron_input.input_mode = InputMode::Normal;
                                app.name_input.reset_cursor();
                                app.cron_input.reset_cursor();
                                app.current_state = State::NotEditing;
                            }
                            _ => {}
                        }
                    }
                    State::DeletingJob => match key.code {
                        KeyCode::Char('y') => {
                            app.delete_job();
                            app.current_state = State::NotEditing;
                        }
                        KeyCode::Char('n') => {
                            app.current_state = State::NotEditing;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

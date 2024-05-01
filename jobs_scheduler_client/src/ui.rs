use crate::app::{App, InputMode, InputSwitch, State};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create main block for title
    let main_block = Block::default()
        .borders(Borders::NONE)
        .title("Job Scheduler");
    let main_chunk = main_block.inner(f.size());
    f.render_widget(main_block, f.size());

    // Divide into 3 layouts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(4),
            ]
            .as_ref(),
        )
        .split(main_chunk);
    let core_chunk = chunks[0];
    let editor_chunk = chunks[1];
    let footer_chunk = chunks[2];

    // Create two chunks for side by side lists
    let lists_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(core_chunk);

    // Draw differents parts of the app
    draw_job_list(f, app, lists_chunks[0]);
    draw_history_list(f, app, lists_chunks[1]);
    draw_editor(f, app, editor_chunk);
    draw_footer(f, app, footer_chunk);
}

fn draw_job_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    // Iterate through all elements in the `items` app
    let items: Vec<ListItem> = app
        .jobs
        .items
        .iter()
        .map(|i| {
            let lines = vec![Line::from(i.0.clone())];
            ListItem::new(lines).style(Style::default().fg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Jobs"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, area, &mut app.jobs.state);
}

fn draw_history_list<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    // The event list doesn't have any state and only displays the current state of the list.
    let events: Vec<ListItem> = app
        .history
        .iter()
        .rev()
        .filter(|history_line| {
            if let Some(job) = app.get_selected_job() {
                history_line.name == job.0
            } else {
                false
            }
        })
        .map(|history_statement| {
            let status = &history_statement.status;
            // Colorcode the level depending on its type
            let s = match status.as_str() {
                "SUCCESS" => Style::default().fg(Color::Blue),
                "ERROR" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };
            let line = Line::from(vec![
                Span::styled(format!("{status:<9}"), s),
                " ".into(),
                Span::styled(
                    history_statement.timestamp.to_rfc2822(),
                    Style::default().italic(),
                ),
                "  ".into(),
                history_statement.error_message.clone().into(),
            ]);

            ListItem::new(vec![line])
        })
        .collect();
    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .start_corner(Corner::TopLeft);
    f.render_widget(events_list, area);
}
fn draw_editor<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    // Create a chuk for each input
    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    // Actualize inputs with selected job if not editing
    if app.current_state == State::NotEditing {
        app.name_input.input = app.get_selected_job_as_strings().0;
        app.cron_input.input = app.get_selected_job_as_strings().1;
    }

    // Create inputs widgets
    let name_input = Paragraph::new(app.name_input.input.as_str())
        .style(match app.name_input.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
            InputMode::Error => Style::default().fg(Color::Red),
        })
        .block(Block::default().borders(Borders::ALL).title("Name"));

    let cron_input = Paragraph::new(app.cron_input.input.as_str())
        .style(match app.cron_input.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
            InputMode::Error => Style::default().fg(Color::Red),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Cron & Command"),
        );

    // Render widgets
    f.render_widget(name_input, input_chunks[0]);
    f.render_widget(cron_input, input_chunks[1]);

    // Make the cursor visible
    match app.current_state {
        State::EditingJob => f.set_cursor(
            input_chunks[1].x + app.cron_input.cursor_position as u16 + 1,
            input_chunks[1].y + 1,
        ),
        State::AddingJob(ref input_switch_value) => {
            match input_switch_value {
                InputSwitch::NameInput => f.set_cursor(
                    input_chunks[0].x + app.name_input.cursor_position as u16 + 1,
                    input_chunks[0].y + 1,
                ),
                InputSwitch::CronInput => f.set_cursor(
                    input_chunks[1].x + app.cron_input.cursor_position as u16 + 1,
                    input_chunks[1].y + 1,
                ),
            };
        }
        _ => {}
    };
}
fn draw_footer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let text = match app.current_state {
        State::NotEditing => vec![
            text::Line::from(vec![
                Span::from("Edit: Enter"),
                Span::raw("  "),
                Span::from("Add: a"),
                Span::raw("  "),
                Span::from("Delete: d"),
            ]),
            text::Line::from(vec![
                Span::from("Move: Up/Down Arrows"),
                Span::raw("  "),
                Span::from("Quit: Escape"),
            ]),
        ],
        State::EditingJob => vec![text::Line::from(vec![
            Span::from("Validate: Enter"),
            Span::raw("  "),
            Span::from("Unvalidate: Escape"),
        ])],
        State::AddingJob(_) => vec![
            text::Line::from(vec![Span::from("Switch input: Tab")]),
            text::Line::from(vec![
                Span::from("Validate: Enter"),
                Span::raw("  "),
                Span::from("Quit: Escape"),
            ]),
        ],
        State::DeletingJob => vec![
            text::Line::from(vec![Span::from(format!(
                "Are you sure to delete {} job",
                app.get_selected_job_as_strings().0
            ))]),
            text::Line::from(vec![
                Span::from("Yes: y"),
                Span::raw("  "),
                Span::from("No: n"),
            ]),
        ],
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Controls", Style::default()));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

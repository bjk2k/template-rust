use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

pub const API_KEY_LEN: usize = 32;

#[derive(Debug, Copy, Clone)]
struct Model {
    counter: i32,
    running_state: RunningState,
    api_key: [u8; API_KEY_LEN],
}

impl Model {
    pub fn new(api_key: [u8; API_KEY_LEN]) -> Self {
        Self {
            api_key,
            running_state: RunningState::default(),
            counter: 0,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq, Debug)]
enum Message {
    Increment,
    Decrement,
    Reset,
    Quit,
}

/// Run the application loop, updating and rendering the TUI.
///
/// # Panics
///
/// Panics if the terminal cannot be initialized or restored.
///
/// # Errors
///
/// This function will return an error if .
pub fn application_loop(api_key: [u8; API_KEY_LEN]) -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;
    let mut model = Model::new(api_key);

    while model.running_state != RunningState::Done {
        // Render the current view
        terminal.draw(|f| view(&mut model, f))?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&model)?;

        // Process updates as long as they return a non-None message
        while current_msg.is_some() {
            // update the model and get the next message
            let (new_model, next_msg) = update(&model, current_msg.unwrap());
            model = new_model;
            current_msg = next_msg;
        }
    }

    tui::restore_terminal()?;
    Ok(())
}

fn view(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    for i in 0..2 {
        let block = Block::bordered().title(format!("Block {}", i));

        frame.render_widget(
            Paragraph::new(format!(
                "Counter: {}, API_KEY: {:?}",
                model.counter, model.api_key
            ))
            .block(block),
            layout[i],
        );
    }
}

/// Convert Event to Message
///
/// We don't need to pass in a `model` to this function in this example
/// but you might need it as your project evolves
fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::Increment),
        KeyCode::Char('k') => Some(Message::Decrement),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &Model, msg: Message) -> (Model, Option<Message>) {
    match msg {
        Message::Increment => {
            let new_model = Model {
                counter: model.counter + 1,
                ..*model
            };
            if model.counter > 50 {
                return (new_model, Some(Message::Reset));
            }
            return (new_model, None);
        }
        Message::Decrement => {
            let new_model = Model {
                counter: model.counter - 1,
                ..*model
            };
            if model.counter < -50 {
                return (new_model, Some(Message::Reset));
            }
            return (new_model, None);
        }
        Message::Reset => {
            let new_model = Model {
                counter: 0,
                ..*model
            };
            return (new_model, None);
        }
        Message::Quit => {
            // You can handle cleanup and exit here
            let new_model = Model {
                running_state: RunningState::Done,
                ..*model
            };
            return (new_model, None);
        }
    };
}

mod tui {
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        Terminal,
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}

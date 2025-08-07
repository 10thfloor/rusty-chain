use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
use std::{
    error::Error,
    io::{self, Write},
};
use tokio::sync::mpsc;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

mod blockchain;
mod p2p;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

enum InputMode {
    Normal,
    Editing,
}

struct App {
    input: String,
    input_mode: InputMode,
    messages: Vec<String>,
    chain: blockchain::Chain,
    p2p: p2p::P2p,
    menu: StatefulList<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <port>", args[0]);
        return Ok(());
    }
    let port = args[1].parse::<u16>().unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (p2p_tx, mut p2p_rx) = mpsc::channel(100);
    let p2p = p2p::P2p::new(port, vec![]).await?;

    let (miner_address, difficulty, token_name, token_symbol) = get_initial_setup(&mut terminal).await?;

    let menu_items = vec![
        "New Transaction".to_string(),
        "Mine Block".to_string(),
        "Create Account".to_string(),
        "Check Balance".to_string(),
    ];
    let mut app = App {
        input: String::new(),
        input_mode: InputMode::Normal,
        messages: Vec::new(),
        chain: blockchain::Chain::new(
            miner_address,
            difficulty,
            token_name,
            token_symbol,
            p2p_tx.clone(),
        ),
        p2p,
        menu: StatefulList::with_items(menu_items),
    };

    let res = run_app(&mut terminal, &mut app, &mut p2p_rx).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    p2p_rx: &mut mpsc::Receiver<p2p::P2pMessage>,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(chunks[0]);

            let menu_items: Vec<ListItem> = app
                .menu
                .items
                .iter()
                .map(|i| ListItem::new(i.clone()))
                .collect();
            let menu = List::new(menu_items)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            f.render_stateful_widget(menu, left_chunks[0], &mut app.menu.state);

            let input = Paragraph::new(app.input.as_ref())
                .style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, left_chunks[1]);

            let messages: Vec<ListItem> = app
                .messages
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let content = format!("{}: {}", i, m);
                    ListItem::new(content)
                })
                .collect();
            let messages =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
            f.render_widget(messages, chunks[1]);
        })?;

        tokio::select! {
            key_event = tokio::task::spawn_blocking(event::read) => {
                if let Ok(Event::Key(key)) = key_event? {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Down => {
                                app.menu.next();
                            }
                            KeyCode::Up => {
                                app.menu.previous();
                            }
                            KeyCode::Enter => {
                                app.input_mode = InputMode::Editing;
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                let selected = app.menu.state.selected().unwrap_or(0);
                                let action = app.menu.items[selected].clone();
                                match action.as_str() {
                                    "New Transaction" => {
                                        app.messages.push("New Transaction selected".to_string());
                                    }
                                    "Mine Block" => {
                                        app.chain.generate_new_block();
                                        app.messages.push("New block mined".to_string());
                                    }
                                    "Create Account" => {
                                        app.messages.push("Create Account selected".to_string());
                                    }
                                    "Check Balance" => {
                                        app.messages.push("Check Balance selected".to_string());
                                    }
                                    _ => {}
                                }
                                app.input.clear();
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
            }
            Some(p2p_message) = p2p_rx.recv() => {
                app.messages.push(format!("{:?}", p2p_message));
            }
        }
    }
}

async fn get_initial_setup<B: Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(String, u32, String, String), Box<dyn Error>> {
    let mut miner_address = String::new();
    let mut difficulty_str = String::new();
    let mut token_name = String::new();
    let mut token_symbol = String::new();

    let mut input = String::new();

    macro_rules! get_input {
        ($prompt:expr, $var:ident) => {
            loop {
                terminal.draw(|f| {
                    let size = f.size();
                    let input_panel = Paragraph::new(format!("{}\n> {}", $prompt, input))
                        .block(Block::default().borders(Borders::ALL).title("Initial Setup"));
                    f.render_widget(input_panel, size);
                })?;
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            $var = input.clone();
                            input.clear();
                            break;
                        }
                        _ => {}
                    }
                }
            }
        };
    }

    get_input!("Enter miner address:", miner_address);
    get_input!("Enter difficulty:", difficulty_str);
    get_input!("Enter token name:", token_name);
    get_input!("Enter token symbol:", token_symbol);

    let difficulty = difficulty_str.trim().parse::<u32>()?;

    Ok((miner_address, difficulty, token_name, token_symbol))
}

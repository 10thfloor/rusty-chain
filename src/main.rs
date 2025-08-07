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
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

mod blockchain;
mod p2p;

struct App {
    input: String,
    messages: Vec<String>,
    chain: blockchain::Chain,
    p2p: p2p::P2p,
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

    let mut app = App {
        input: String::new(),
        messages: Vec::new(),
        chain: blockchain::Chain::new(
            "miner_address".to_string(), // Dummy address
            1,                          // Dummy difficulty
            "TestCoin".to_string(),
            "TSC".to_string(),
            p2p_tx.clone(),
        ),
        p2p,
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

            let input = Paragraph::new(app.input.as_ref())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[0]);

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
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            app.messages.push(app.input.drain(..).collect());
                        }
                        _ => {}
                    }
                }
            }
            Some(p2p_message) = p2p_rx.recv() => {
                app.messages.push(format!("{:?}", p2p_message));
            }
        }
    }
}

fn handle_new_transaction(chain: &mut blockchain::Chain) -> io::Result<bool> {
    let mut sender = String::new();
    let mut receiver = String::new();
    let mut amount_str = String::new();

    print!("Enter sender account: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut sender)?;

    let amount = loop {
        print!("Enter amount: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut amount_str)?;
        match amount_str.trim().parse::<i64>() {
            Ok(a) => break a,
            Err(_) => {
                println!("Invalid input, please enter an integer.");
                amount_str.clear();
                continue;
            }
        }
    };

    print!("Enter receiving account: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut receiver)?;

    Ok(chain.new_transaction(
        sender.trim().to_string(),
        receiver.trim().to_string(),
        amount,
    ))
}

fn handle_create_account(chain: &mut blockchain::Chain) -> io::Result<bool> {
    let mut account = String::new();
    print!("Enter account name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut account)?;
    Ok(chain.create_account(account.trim().to_string()))
}

fn handle_check_balance(chain: &blockchain::Chain) -> io::Result<()> {
    let mut account = String::new();
    print!("Enter account name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut account)?;
    match chain.get_balance(&account.trim().to_string()) {
        Some(balance) => println!("Balance: {}", balance),
        None => println!("Account not found."),
    }
    Ok(())
}

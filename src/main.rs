use std::io::{self, Write};
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;

mod blockchain;
mod p2p;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <port>", args[0]);
        return Ok(());
    }
    let port = args[1].parse::<u16>().unwrap();

    let (tx, mut rx) = mpsc::channel(100);
    let p2p_tx = tx.clone();

    let mut p2p = p2p::P2p::new(port, vec![]).await.unwrap();
    let p2p_handle = tokio::spawn(async move {
        p2p.run(p2p_tx).await;
    });

    let mut miner_addr = String::new();
    let mut difficulty_str = String::new();

    // Get miner address
    print!("Input a miner address: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut miner_addr)?;

    // Get difficulty
    let difficulty = loop {
        print!("Difficulty: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut difficulty_str)?;
        match difficulty_str.trim().parse::<u32>() {
            Ok(d) => break d,
            Err(_) => {
                println!("Invalid input, please enter an integer.");
                difficulty_str.clear();
                continue;
            }
        }
    };

    // Get token name
    let mut token_name = String::new();
    print!("Enter token name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut token_name)?;

    // Get token symbol
    let mut token_symbol = String::new();
    print!("Enter token symbol: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut token_symbol)?;

    // The chain
    let mut chain = blockchain::Chain::new(
        miner_addr.trim().to_string(),
        difficulty,
        token_name.trim().to_string(),
        token_symbol.trim().to_string(),
        tx.clone(),
    );

    loop {
        println!("\n -- Menu -- ");
        println!("(1) New Transaction");
        println!("(2) Mine block");
        println!("(3) Create account");
        println!("(4) Check balance");
        println!("(5) Change difficulty");
        println!("(6) Change reward");
        println!("(0) Exit");
        print!("Enter your choice ~> ");
        io::stdout().flush()?;

        let mut choice = String::new();
        tokio::select! {
            _ = async {
                let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
                stdin.read_line(&mut choice).await.unwrap();
            } => {
                match choice.trim().parse().unwrap_or(99) {
                    0 => {
                        println!("Goodbye.");
                        break;
                    }
                    1 => {
                        if handle_new_transaction(&mut chain)? {
                            println!("Transaction was added!");
                        } else {
                            println!("Transaction failed :(");
                        }
                    }
                    2 => {
                        println!("Generating block ...");
                        if chain.generate_new_block() {
                            println!("Block was generated.");
                        } else {
                            println!("Block generation failed :(");
                        }
                    }
                    3 => {
                        if handle_create_account(&mut chain)? {
                            println!("Account was created!");
                        } else {
                            println!("Account creation failed :(");
                        }
                    }
                    4 => {
                        handle_check_balance(&chain)?;
                    }
                    5 => println!("Not implemented."),
                    6 => println!("Not implemented."),
                    _ => println!("Invalid input."),
                }
            }
            Some(message) = rx.recv() => {
                match message.message {
                    p2p::Message::NewBlock(block) => {
                        if chain.resolve_conflict(&[block]) {
                            println!("New block received and chain updated.");
                        }
                    }
                    p2p::Message::NewTransaction(tx) => {
                        chain.new_transaction(tx.sender, tx.receiver, tx.amount);
                        println!("New transaction received.");
                    }
                    p2p::Message::GetBlocks(addr) => {
                        // This is a simplified implementation. A real implementation would
                        // send the blocks to the requesting peer.
                        println!("Received GetBlocks request from {}", addr);
                    }
                    p2p::Message::Blocks(blocks) => {
                        if chain.resolve_conflict(&blocks) {
                            println!("Blocks received and chain updated.");
                        }
                    }
                }
            }
        }
    }

    p2p_handle.await?;
    Ok(())
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

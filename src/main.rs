use std::io::{self, Write};

mod blockchain;

fn main() -> io::Result<()> {
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
    );

    loop {
        println!("\n -- Menu -- ");
        println!("(1) New Transaction");
        println!("(2) Mine block");
        println!("(3) Change difficulty");
        println!("(4) Change reward");
        println!("(0) Exit");
        print!("Enter your choice ~> ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

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
            3 => println!("Not implemented."),
            4 => println!("Not implemented."),
            _ => println!("Invalid input."),
        }
    }

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

#[macro_use]
extern crate serde_derive;

use std::io;
use std::io::Write;
use std::process;

mod blockchain;

fn main() {
    let mut miner_addr = String::new();
    let mut difficulty = String::new();
    let mut choice = String::new();

    print!("Input a miner address: ");
    io::stdout().flush();
    io::stdin().read_line(&mut miner_addr);
    print!("Difficulty: ");
    io::stdout().flush();
    io::stdin().read_line(&mut difficulty);
    let diff = difficulty
        .trim()
        .parse::<u32>()
        .expect("Input was not an integer.");

    // The chain
    let mut chain = blockchain::Chain::new(miner_addr.trim().to_string(), diff);

    loop {
        print!(" -- Menu -- ");
        print!("(1) New Transaction ");
        print!("(2) Mine block ");
        print!("(3) Change difficulty ");
        print!("(4) Change reward ");
        print!("(0) Exit ");
        print!(" Enter your choice ~> ");
        io::stdout().flush();
        choice.clear();
        io::stdin().read_line(&mut choice);
        print!("");

        match choice.trim().parse().unwrap() {
            0 => {
                println!("Goodbye.");
                process::exit(0);
            }
            1 => {
                let mut sender = String::new();
                let mut receiver = String::new();
                let mut amount = String::new();

                print!("Enter sender account: ");
                io::stdout().flush();
                io::stdin().read_line(&mut sender);
                print!("Enter amount: ");
                io::stdout().flush();
                io::stdin().read_line(&mut amount);
                print!("Enter receiving account: ");
                io::stdout().flush();
                io::stdin().read_line(&mut receiver);

                let result = chain.new_transaction(
                    sender.trim().to_string(),
                    receiver.trim().to_string(),
                    amount.trim().parse().unwrap(),
                );

                match result {
                    true => println!("Transaction was added!"),
                    false => println!("Transaction failed :("),
                }
            }
            2 => {
                println!("Generating block ...");
                let result = chain.generate_new_block();
                match result {
                    true => println!("Block was generated."),
                    false => println!("Block generation failed :("),
                }
            }
            3 => println!("Not implemented."),
            4 => println!("Not implemented."),
            _ => println!("Invalid input."),
        }
    }
}

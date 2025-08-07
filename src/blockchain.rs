use chrono::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

const REWARD: i64 = 420;
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: i64,
}

#[derive(Debug, Serialize)]
pub struct BlockHeader {
    timestamp: i64,
    nonce: u32,
    previous_hash: String,
    merkle: String,
    difficulty: u32,
}

#[derive(Debug, Serialize)]
pub struct Block {
    header: BlockHeader,
    count: u32,
    transactions: Vec<Transaction>,
}

pub struct Chain {
    chain: Vec<Block>,
    current_transaction: Vec<Transaction>,
    difficulty: u32,
    miner_address: String,
    reward: i64,
}

impl Chain {
    pub fn new(miner_address: String, difficulty: u32) -> Chain {
        let mut chain = Chain {
            chain: Vec::new(),
            current_transaction: Vec::new(),
            difficulty,
            miner_address,
            reward: REWARD,
        };

        chain.generate_new_block();
        chain
    }

    pub fn new_transaction(&mut self, sender: String, receiver: String, amount: i64) -> bool {
        self.current_transaction.push(Transaction {
            sender,
            receiver,
            amount,
        });
        true
    }

    pub fn last_hash(&self) -> String {
        self.chain
            .last()
            .map(|block| Chain::hash(&block.header).expect("Failed to hash block header"))
            .unwrap_or_else(|| GENESIS_HASH.to_string())
    }

    pub fn update_difficulty(&mut self, difficulty: u32) -> bool {
        self.difficulty = difficulty;
        true
    }

    pub fn update_reward(&mut self, reward: i64) -> bool {
        self.reward = reward;
        true
    }

    pub fn generate_new_block(&mut self) -> bool {
        let header = BlockHeader {
            timestamp: Utc::now().timestamp_millis(),
            nonce: 0,
            previous_hash: self.last_hash(),
            difficulty: self.difficulty,
            merkle: String::new(),
        };

        let reward_transaction = Transaction {
            sender: String::from("Root"),
            receiver: self.miner_address.clone(),
            amount: self.reward,
        };

        let mut block = Block {
            header,
            count: 0,
            transactions: vec![],
        };

        block.transactions.push(reward_transaction);
        block.transactions.append(&mut self.current_transaction);
        block.count = block.transactions.len() as u32;
        block.header.merkle =
            Chain::get_merkle(block.transactions.clone()).expect("Failed to calculate Merkle root");
        Chain::proof_of_work(&mut block.header);
        println!("{:?}", &block);
        self.chain.push(block);
        true
    }

    fn get_merkle(transactions: Vec<Transaction>) -> Result<String, serde_json::Error> {
        let mut merkle = Vec::new();
        for t in &transactions {
            let hash = Chain::hash(t)?;
            merkle.push(hash);
        }

        if merkle.is_empty() {
            return Ok(String::new());
        }
        if merkle.len() % 2 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last);
        }

        while merkle.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in merkle.chunks(2) {
                let h1 = chunk[0].clone();
                let h2 = chunk.get(1).cloned().unwrap_or_else(|| h1.clone());
                let combined = format!("{}{}", h1, h2);
                let new_hash = Chain::hash(&combined)?;
                next_level.push(new_hash);
            }
            merkle = next_level;
        }

        Ok(merkle.pop().unwrap())
    }

    pub fn proof_of_work(header: &mut BlockHeader) {
        let prefix = "0".repeat(header.difficulty as usize);
        loop {
            let hash = Chain::hash(header).expect("Failed to hash header");
            if hash.starts_with(&prefix) {
                println!("Block hash: {}", hash);
                break;
            }
            header.nonce += 1;
        }
    }

    pub fn hash<T: serde::Serialize>(item: &T) -> Result<String, serde_json::Error> {
        let input = serde_json::to_string(item)?;
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let res = hasher.finalize();
        Ok(hex::encode(res))
    }
}

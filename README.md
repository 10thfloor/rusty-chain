# Toy Blockchain in Rust

This project is a simplified blockchain implementation in Rust, created for educational purposes. It's a command-line application that allows you to create a blockchain, mine new blocks, create accounts, and transfer tokens between them. It's a great way to learn about the fundamental concepts of blockchain technology in a hands-on way.

## Features

*   **Create a new blockchain**: Initialize a new blockchain with a custom token name and symbol.
*   **Mine new blocks**: Generate new blocks through a proof-of-work algorithm.
*   **Create accounts**: Create new accounts to send and receive tokens.
*   **Transfer tokens**: Transfer tokens between accounts.
*   **Check balances**: Check the token balance of any account.

## How to Use

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-username/toy-blockchain-rust.git
    cd toy-blockchain-rust
    ```

2.  **Compile and run the application**:
    ```bash
    cargo run
    ```

3.  **Follow the on-screen prompts**:
    The application will guide you through the process of creating a new blockchain, creating accounts, and sending tokens. Here is an example of the menu you will see:
    ```
     -- Menu --
    (1) New Transaction
    (2) Mine block
    (3) Create account
    (4) Check balance
    (5) Change difficulty
    (6) Change reward
    (0) Exit
    Enter your choice ~>
    ```

### Running the P2P Network

To run the application as a node in a P2P network, you need to specify a port for it to listen on. You can also provide a list of peer addresses to connect to.

**Terminal 1:**
```bash
cargo run 8080
```

**Terminal 2:**
```bash
cargo run 8081 127.0.0.1:8080
```

This will start two nodes, with the second node connecting to the first. You can then create a transaction on one node and see it propagate to the other.

## Learning Concepts

This project is a great way to learn about the following blockchain concepts:

*   **Blocks and Chains**: Understand how blocks are created and linked together to form a chain.
*   **Proof of Work**: See a simple implementation of a proof-of-work algorithm to secure the blockchain.
*   **Merkle Trees**: Learn how Merkle trees are used to summarize the transactions in a block.
*   **Transactions**: Understand how transactions are created and added to blocks.
*   **Account Balances**: See how account balances are tracked and updated in a blockchain system.

## Future Improvements

This project can be extended with the following features:

*   **Public/Private Key Cryptography**: Implement a more robust wallet system using public/private key pairs to sign and verify transactions.
*   **Network Layer**: Add a networking layer to allow multiple nodes to connect and participate in the blockchain.
*   **Peer-to-Peer Communication**: Implement a peer-to-peer communication protocol for nodes to share information about new blocks and transactions.
*   **Consensus Algorithm**: Implement a more advanced consensus algorithm, such as Proof of Stake.

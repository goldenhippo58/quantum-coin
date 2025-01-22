mod blockchain;
mod rest;
mod wallet;
mod network {
    pub mod p2p; // Import the p2p module from the network directory
}

use std::sync::{Arc, Mutex};
use tokio::task;

#[tokio::main]
async fn main() {
    let blockchain = Arc::new(Mutex::new(blockchain::chain::Blockchain::new(
        "./blockchain_data",
    )));

    // Start the REST API in a separate task
    let blockchain_api = blockchain.clone();
    task::spawn(async move {
        println!("Starting REST API on http://localhost:8080...");
        rest::start_rest_api(blockchain_api).await;
    });

    // Start the P2P network node
    println!("Starting P2P Node on port 8081...");
    network::p2p::start_node(8081).await;
}

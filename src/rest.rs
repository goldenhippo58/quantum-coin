use crate::blockchain::chain::Blockchain;
use crate::wallet;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE; // Base64 Engine
use base64::Engine;
use rocksdb::DB;
use std::sync::{Arc, Mutex};
use warp::Filter;

type SharedBlockchain = Arc<Mutex<Blockchain>>;

pub async fn start_rest_api(blockchain: SharedBlockchain) {
    // Wallet creation
    let wallet_create = warp::path!("wallet" / "create").and(warp::post()).map(|| {
        let (public_key, _) = wallet::create_wallet();
        warp::reply::json(&serde_json::json!({
            "message": "Wallet created successfully!",
            "public_key": public_key
        }))
    });

    // Sending transactions
    let wallet_send = warp::path!("wallet" / "send")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            let sender = body["sender"].as_str().unwrap_or_default();
            let to = body["to"].as_str().unwrap_or_default();
            let amount = body["amount"].as_u64().unwrap_or(0);

            wallet::send_transaction(blockchain, sender, to, amount);
            warp::reply::json(&serde_json::json!({
                "message": "Transaction sent!",
                "sender": sender,
                "to": to,
                "amount": amount
            }))
        });

    // Staking
    let stake = warp::path!("stake")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            let wallet = body["wallet"].as_str().unwrap_or_default().to_string();
            let amount = body["amount"].as_u64().unwrap_or(0);

            let mut blockchain = blockchain.lock().unwrap();
            blockchain.stake(wallet.clone(), amount);

            warp::reply::json(&serde_json::json!({
                "message": "Stake successful",
                "wallet": wallet,
                "amount": amount
            }))
        });

    // Unstaking
    let unstake = warp::path!("unstake")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            let wallet = body["wallet"].as_str().unwrap_or_default().to_string();
            let amount = body["amount"].as_u64().unwrap_or(0);

            let mut blockchain = blockchain.lock().unwrap();
            let success = blockchain.unstake(wallet.clone(), amount);

            warp::reply::json(&serde_json::json!({
                "message": if success { "Unstake successful" } else { "Unstake failed" },
                "wallet": wallet,
                "amount": amount
            }))
        });

    // Checking staking rewards
    let rewards = warp::path!("staking" / "rewards")
        .and(warp::get())
        .and(warp::query::<serde_json::Value>())
        .and(with_blockchain(blockchain.clone()))
        .map(|query: serde_json::Value, blockchain: SharedBlockchain| {
            let wallet = query["wallet"].as_str().unwrap_or_default().to_string();

            let blockchain = blockchain.lock().unwrap();
            let rewards = blockchain.get_rewards(&wallet);

            warp::reply::json(&serde_json::json!({
                "wallet": wallet,
                "rewards": rewards
            }))
        });

    // Check all wallets
    let wallets_route = warp::path("wallets")
        .and(warp::get())
        .map(|| {
            let db = DB::open_default("./wallets").expect("Failed to open wallet database");
            let wallets: Vec<String> = db
                .iterator(rocksdb::IteratorMode::Start)
                .map(|item| match item {
                    Ok((key, _)) => String::from_utf8(key.to_vec()).unwrap(),
                    Err(_) => "Invalid key".to_string(),
                })
                .collect();
            warp::reply::json(&serde_json::json!({ "wallets": wallets }))
        });

    // Check all blocks
    let blocks_route = warp::path("blocks")
        .and(warp::get())
        .and(with_blockchain(blockchain.clone()))
        .map(|blockchain: SharedBlockchain| {
            let chain = blockchain.lock().unwrap();
            warp::reply::json(&chain.chain)
        });

    // Check wallet transactions
    let transactions = warp::path!("wallet" / "transactions")
        .and(warp::get())
        .and(warp::query::<serde_json::Value>())
        .and(with_blockchain(blockchain.clone()))
        .map(|query: serde_json::Value, blockchain: SharedBlockchain| {
            let wallet = query["wallet"].as_str().unwrap_or_default().to_string();
            let blockchain = blockchain.lock().unwrap();

            let txs: Vec<_> = blockchain.chain.iter()
                .flat_map(|block| block.transactions.iter())
                .filter(|tx| tx.sender == wallet || tx.receiver == wallet)
                .cloned()
                .collect();

            warp::reply::json(&txs)
        });

    // Get wallet balance
    let wallet_balance = warp::path!("wallet" / "balance")
        .and(warp::get())
        .and(warp::query::<serde_json::Value>())
        .and(with_blockchain(blockchain.clone()))
        .map(|query: serde_json::Value, blockchain: SharedBlockchain| {
            let wallet = query["wallet"].as_str().unwrap_or_default().to_string();
            let blockchain = blockchain.lock().unwrap();

            let spendable = blockchain.chain.iter()
                .flat_map(|block| block.transactions.iter())
                .fold(0, |balance, tx| {
                    if tx.sender == wallet {
                        balance - tx.amount
                    } else if tx.receiver == wallet {
                        balance + tx.amount
                    } else {
                        balance
                    }
                });

            let staked = blockchain.staking_state.staked_balances.get(&wallet).cloned().unwrap_or(0);

            warp::reply::json(&serde_json::json!({
                "wallet": wallet,
                "spendable_balance": spendable,
                "staked_balance": staked,
                "total_balance": spendable + staked,
            }))
        });

    // Add Task
    let add_task = warp::path!("tasks" / "add")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            let id = body["id"].as_str().unwrap_or_default().to_string();
            let description = body["description"].as_str().unwrap_or_default().to_string();
            let data = BASE64_ENGINE.decode(body["data"].as_str().unwrap_or_default()).unwrap_or_default();
            let reward = body["reward"].as_u64().unwrap_or(0);

            let mut blockchain = blockchain.lock().unwrap();
            blockchain.add_task(id, description, data, reward);

            warp::reply::json(&serde_json::json!({ "message": "Task added successfully" }))
        });

    // Complete Task
    let complete_task = warp::path!("tasks" / "complete")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            let id = body["id"].as_str().unwrap_or_default().to_string();

            let mut blockchain = blockchain.lock().unwrap();
            if let Some(task) = blockchain.complete_task(&id) {
                warp::reply::json(&serde_json::json!({
                    "message": "Task completed successfully",
                    "task": task
                }))
            } else {
                warp::reply::json(&serde_json::json!({ "error": "Task not found" }))
            }
        });

    // Get Pending Tasks
    let get_pending_tasks = warp::path!("tasks" / "pending")
        .and(warp::get())
        .and(with_blockchain(blockchain.clone()))
        .map(|blockchain: SharedBlockchain| {
            let blockchain = blockchain.lock().unwrap();
            let tasks: Vec<_> = blockchain.get_pending_tasks()
                .into_iter()
                .map(|t| serde_json::json!({
                    "id": t.id,
                    "description": t.description,
                    "reward": t.reward
                }))
                .collect();

            warp::reply::json(&tasks)
        });

    // Get Compelted Tasks
    let get_completed_tasks = warp::path!("tasks" / "completed")
        .and(warp::get())
        .and(with_blockchain(blockchain.clone()))
        .map(|blockchain: SharedBlockchain| {
            let blockchain = blockchain.lock().unwrap();
            let tasks: Vec<_> = blockchain
                .task_queue
                .tasks
                .values()
                .filter(|task| task.is_complete)
                .map(|task| serde_json::json!({
                    "id": task.id,
                    "description": task.description,
                    "reward": task.reward,
                    "is_complete": task.is_complete
                }))
                .collect();
            warp::reply::json(&tasks)
        });


    // Run the server
    warp::serve(
        wallet_create
            .or(wallet_send)
            .or(stake)
            .or(unstake)
            .or(rewards)
            .or(wallets_route)
            .or(blocks_route)
            .or(transactions)
            .or(wallet_balance)
            .or(add_task)
            .or(complete_task)
            .or(get_pending_tasks),
    )
    .run(([0, 0, 0, 0], 8080))
    .await;
}

fn with_blockchain(
    blockchain: SharedBlockchain,
) -> impl Filter<Extract = (SharedBlockchain,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || blockchain.clone())
}

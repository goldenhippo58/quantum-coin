use crate::blockchain::chain::Blockchain;
use crate::cryptography::kyber::KyberEncryption;
use crate::cryptography::sphincs::Sphincs;
use crate::wallet;
use hex;
use pqcrypto_kyber::kyber512::{Ciphertext, SecretKey};
use pqcrypto_sphincsplus::sphincssha2128fsimple::{
    self, verify_detached_signature, PublicKey as SphincsPublicKey,
};

use pqcrypto_traits::{
    kem::{Ciphertext as _, PublicKey as _, SecretKey as _, SharedSecret as _},
    sign::{DetachedSignature as _, PublicKey as _},
};
use rocksdb::DB;
use serde_json;
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
    let wallets_route = warp::path("wallets").and(warp::get()).map(|| {
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

            let txs: Vec<_> = blockchain
                .chain
                .iter()
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

            let spendable = blockchain
                .chain
                .iter()
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

            let staked = blockchain
                .staking_state
                .staked_balances
                .get(&wallet)
                .cloned()
                .unwrap_or(0);

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
            println!("Received request to add task: {:?}", body);

            // Extract task data
            let id = body["id"].as_str().unwrap_or_default().to_string();
            let description = body["description"].as_str().unwrap_or_default().to_string();
            let _data = body["data"].as_str().unwrap_or_default().as_bytes(); // Prefix with `_` to suppress warning
            let reward = body["reward"].as_u64().unwrap_or(0);

            // Kyber: Generate key pair and encrypt data
            let (public_key, secret_key) = KyberEncryption::generate_keypair();
            let (_shared_secret, ciphertext) =
                KyberEncryption::encrypt(&public_key).expect("Encryption failed");

            // SPHINCS+: Generate key pair and sign task metadata
            let (sphincs_public_key, sphincs_secret_key) =
                Sphincs::generate_keypair().expect("Failed to generate SPHINCS+ keypair");
            let task_metadata = format!("{}:{}", id, description);
            let signature = Sphincs::sign(task_metadata.as_bytes(), &sphincs_secret_key)
                .expect("Failed to sign metadata");

            // Convert keys and data to hex for storage
            let secret_key_bytes = SecretKey::as_bytes(&secret_key);
            let secret_key_hex = hex::encode(secret_key_bytes);

            let ciphertext_bytes = Ciphertext::as_bytes(&ciphertext);
            let encrypted_data_hex = hex::encode(ciphertext_bytes);

            let sphincs_public_key_bytes = SphincsPublicKey::as_bytes(&sphincs_public_key);
            let sphincs_public_key_hex = hex::encode(sphincs_public_key_bytes);
            let sphincs_signature_hex = hex::encode(&signature);

            // Store the task in the blockchain
            let mut blockchain = blockchain.lock().unwrap();
            blockchain.add_task(
                id.clone(),
                description.clone(),
                encrypted_data_hex.clone().into_bytes(),
                reward,
                sphincs_public_key_bytes.to_vec(),
                signature.clone(), // Clone here to avoid move errors
            );

            // Respond with task details
            warp::reply::json(&serde_json::json!({
                "message": "Task added successfully",
                "task_id": id,
                "public_key": hex::encode(&public_key.as_bytes()),
                "secret_key": secret_key_hex,
                "sphincs_public_key": sphincs_public_key_hex,
                "sphincs_signature": sphincs_signature_hex,
                "encrypted_data": encrypted_data_hex
            }))
        });

    // Complete Task
    let complete_task = warp::path!("tasks" / "complete")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .map(|body: serde_json::Value, blockchain: SharedBlockchain| {
            println!("Received request to complete task: {:?}", body);

            // Extract task data
            let id = body["id"].as_str().unwrap_or_default().to_string();
            let encrypted_result = body["result"].as_str().unwrap_or_default();
            let secret_key_hex = body["secret_key"].as_str().unwrap_or_default();

            // Decode encrypted result and secret key
            let ciphertext_bytes = hex::decode(encrypted_result).expect("Invalid ciphertext");
            let secret_key_bytes = hex::decode(secret_key_hex).expect("Invalid secret key");

            // Convert bytes back to Kyber types using proper methods
            let ciphertext = KyberEncryption::ciphertext_from_bytes(&ciphertext_bytes)
                .expect("Invalid ciphertext format");
            let secret_key = KyberEncryption::secret_key_from_bytes(&secret_key_bytes)
                .expect("Invalid secret key format");

            // Kyber: Decrypt result
            let shared_secret =
                KyberEncryption::decrypt(&ciphertext, &secret_key).expect("Decryption failed");

            // Get task details from blockchain
            let blockchain_guard = blockchain.lock().unwrap();
            let task = blockchain_guard
                .get_task(&id)
                .expect("Task not found")
                .clone(); // Clone it before dropping the lock
            drop(blockchain_guard); // Now it's safe to release the lock
            let description = task.description.clone();

            // Recreate task metadata for SPHINCS+ signature verification
            let task_metadata = format!("{}:{}", id, description);

            // Decode SPHINCS+ public key and signature from the task
            let sphincs_public_key_bytes =
                hex::decode(&task.sphincs_public_key).expect("Invalid SPHINCS+ public key");
            let sphincs_signature =
                hex::decode(&task.signature).expect("Invalid SPHINCS+ signature");

            // Convert bytes to SPHINCS+ types using proper methods
            let detached_signature =
                sphincssha2128fsimple::DetachedSignature::from_bytes(&sphincs_signature)
                    .expect("Invalid signature format");
            let sphincs_public_key =
                sphincssha2128fsimple::PublicKey::from_bytes(&sphincs_public_key_bytes)
                    .expect("Invalid public key format");

            // Verify SPHINCS+ signature using detached signature
            if !verify_detached_signature(
                &detached_signature,
                &task_metadata.as_bytes(),
                &sphincs_public_key,
            )
            .is_ok()
            {
                return warp::reply::json(&serde_json::json!({
                    "error": "Invalid SPHINCS+ signature"
                }));
            }

            // Mark task as complete in the blockchain
            let mut blockchain = blockchain.lock().unwrap();
            if let Some(task) = blockchain.complete_task(&id) {
                warp::reply::json(&serde_json::json!({
                    "message": "Task completed successfully",
                    "task": task,
                    "decrypted_data": String::from_utf8(shared_secret.as_bytes().to_vec())
                        .unwrap_or_default()
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
            let tasks: Vec<_> = blockchain
                .get_pending_tasks()
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "description": t.description,
                        "reward": t.reward
                    })
                })
                .collect();

            warp::reply::json(&tasks)
        });

    // Get Completed Tasks
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
                .map(|task| {
                    serde_json::json!({
                        "id": task.id,
                        "description": task.description,
                        "reward": task.reward,
                        "is_complete": task.is_complete,
                        "sphincs_public_key": hex::encode(&task.sphincs_public_key),
                        "sphincs_signature": hex::encode(&task.signature)
                    })
                })
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
            .or(get_pending_tasks)
            .or(get_completed_tasks),
    )
    .run(([0, 0, 0, 0], 8080))
    .await;
}

fn with_blockchain(
    blockchain: SharedBlockchain,
) -> impl Filter<Extract = (SharedBlockchain,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || blockchain.clone())
}

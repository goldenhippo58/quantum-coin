mod blockchain;

use blockchain::block::Transaction;
use blockchain::chain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    let tx1 = Transaction {
        sender: "Alice".to_string(),
        receiver: "Bob".to_string(),
        amount: 100,
        signature: vec![],
    };

    blockchain.add_block(vec![tx1]);

    println!("Blockchain: {:?}", blockchain);
    println!("Is valid: {}", blockchain.validate_chain());
}

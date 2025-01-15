use super::tx::Transaction;

pub struct TransactionPool {
    pub transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }
}

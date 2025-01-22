use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StakingState {
    pub staked_balances: HashMap<String, u64>, // Wallet address -> Staked amount
    pub validators: Vec<String>,              // List of validators
    pub total_staked: u64,                    // Total staked amount in the network
}

impl StakingState {
    pub fn new() -> Self {
        Self {
            staked_balances: HashMap::new(),
            validators: Vec::new(),
            total_staked: 0,
        }
    }

    pub fn stake(&mut self, wallet: String, amount: u64) {
        let current_balance = self.staked_balances.entry(wallet.clone()).or_insert(0);
        *current_balance += amount;
        self.total_staked += amount;

        if !self.validators.contains(&wallet) {
            self.validators.push(wallet);
        }
    }

    pub fn unstake(&mut self, wallet: String, amount: u64) -> bool {
        if let Some(balance) = self.staked_balances.get_mut(&wallet) {
            if *balance >= amount {
                *balance -= amount;
                self.total_staked -= amount;

                if *balance == 0 {
                    self.validators.retain(|v| v != &wallet);
                }
                return true;
            }
        }
        false
    }

    pub fn get_rewards(&self, wallet: &String) -> u64 {
        let balance = self.staked_balances.get(wallet).unwrap_or(&0);
        let reward_rate = 0.05; // Example: 5% annualized reward
        (*balance as f64 * reward_rate) as u64
    }
}

#[test]
fn test_staking_logic() {
    let mut blockchain = Blockchain::new("./test_blockchain");

    blockchain.stake("wallet1".to_string(), 100);
    assert_eq!(blockchain.staking_state.total_staked, 100);

    let rewards = blockchain.get_rewards(&"wallet1".to_string());
    assert!(rewards > 0);

    let success = blockchain.unstake("wallet1".to_string(), 50);
    assert!(success);
    assert_eq!(blockchain.staking_state.total_staked, 50);
}

#[test]
fn test_validate_block() {
    let blockchain = Blockchain::new("./test_blockchain");

    let block = Block::new(1, "0".to_string(), vec![]);
    assert!(blockchain.validate_block(&block));
}

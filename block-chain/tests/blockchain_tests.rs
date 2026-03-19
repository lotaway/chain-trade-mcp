use block_chain::{Block, BlockChain, Transaction, Output};

use block_chain::time::now_ms;

#[test]
fn test_create_block() {
    let index = 0;
    let timestamp = now_ms();
    let prev_hash = vec![0; 32];
    let nonce = 0;
    let transactions = Vec::<Transaction>::new();
    let difficulty = 0x0088FFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let mut block = Block::new(index, timestamp, prev_hash.clone(), nonce, transactions.clone(), difficulty);
    
    assert_eq!(block.nonce, nonce);
    block.mine();
    assert_eq!(block.index, index);
    assert_eq!(block.timestamp, timestamp);
    assert_eq!(block.prev_block_hash, prev_hash);
    assert_eq!(block.difficulty, difficulty);
}

#[test]
fn test_create_blockchain() {
    let mut blockchain = BlockChain::new();
    assert!(blockchain.init().is_ok());
    
    let prev_hash = blockchain.last_hash();
    let original_len = blockchain.len();
    let transactions = vec![
        Transaction {
            inputs: Vec::new(),
            outputs: vec![
                Output {
                    to: String::from("miner"),
                    value: 10,
                },
            ],
        },
        Transaction {
            inputs: vec![
                blockchain.blocks[0].transactions[0].outputs[0].clone(),
            ],
            outputs: vec![
                Output {
                    to: String::from("Batman"),
                    value: 20,
                },
                Output {
                    to: String::from("Superman"),
                    value: 20,
                },
            ],
        }
    ];


    let difficulty = 0x0088FFFFFFFFFFFFFFFFFFFFFFFFFFF;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut block = Block::new(original_len, now_ms(), prev_hash, 0, transactions, difficulty);

    block.mine();
    
    blockchain.update_with(block).expect("Failed to update blockchain with new block");
    assert_eq!(original_len + 1, blockchain.len());
    assert!(blockchain.verify_all_blocks().is_ok());
}


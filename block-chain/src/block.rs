use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use crate::hashable::Hashable;
use crate::Transaction;

#[derive(Clone, Deserialize, Serialize)]
pub struct Block {
    pub index: usize,
    pub timestamp: u128,
    pub prev_block_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
    pub difficulty: u128,
}

impl Hashable for Block {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&(self.index as u32).to_le_bytes());
        bytes.extend(&self.timestamp.to_le_bytes());
        bytes.extend(&self.prev_block_hash);
        bytes.extend(&self.nonce.to_le_bytes());
        bytes.extend(self.transactions.bytes());
        bytes.extend(&self.difficulty.to_le_bytes());
        bytes
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block[{}]: {} at: {} with: {} nonce: {}",
            &self.index,
            &hex::encode(&self.hash),
            &self.timestamp,
            &self.transactions.len(),
            &self.nonce
        )
    }
}

impl Block {
    pub fn new(
        index: usize,
        timestamp: u128,
        prev_block_hash: Vec<u8>,
        nonce: u64,
        transactions: Vec<Transaction>,
        difficulty: u128,
    ) -> Self {
        let mut block = Self {
            index,
            timestamp,
            prev_block_hash,
            hash: Vec::new(),
            nonce,
            transactions,
            difficulty,
        };
        block.hash = block.hash();
        block
    }

    pub fn mine(&mut self) {
        for n in 0..u64::MAX {
            self.nonce = n;
            let hash = self.hash();
            if self.verify_difficulty(&hash) {
                self.hash = hash;
                return;
            }
        }
    }

    fn verify_difficulty(&self, hash: &[u8]) -> bool {
        if hash.len() < 32 {
            return false;
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&hash[16..32]);
        let val = u128::from_be_bytes(bytes);
        self.difficulty < val
    }
}

#[derive(Debug)]
pub enum BlockError {
    MismatchedIndex,
    InvalidHash,
    AChronologicalTimestamp,
    MismatchedPreviousHash,
    InvalidGenesisFormat,
    InvalidInput,
    InsufficientInput,
    InvalidCoinbase,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BlockError {}


#[derive(Debug)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
    pub unspent_outputs: HashSet<Vec<u8>>,
}

impl BlockChain {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            unspent_outputs: HashSet::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), BlockError> {
        let genesis_tx = Transaction {
            inputs: Vec::new(),
            outputs: vec![
                crate::transaction::Output { to: "creator".into(), value: 50 },
                crate::transaction::Output { to: "genesis".into(), value: 25 },
            ],
        };
        let genesis_block = Block::new(
            0,
            crate::time::now_ms(),
            vec![0; 32],
            0,
            vec![genesis_tx],
            0x000FFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        );
        self.update_with(genesis_block)
    }

    pub fn update_with(&mut self, block: Block) -> Result<(), BlockError> {
        self.verify_block(&block, self.blocks.len())?;
        self.process_transactions(&block)?;
        self.blocks.push(block);
        Ok(())
    }

    fn verify_block(&self, block: &Block, expected_index: usize) -> Result<(), BlockError> {
        if block.index != expected_index {
            return Err(BlockError::MismatchedIndex);
        }
        if !block.verify_difficulty(&block.hash) {
            return Err(BlockError::InvalidHash);
        }
        if block.index > 0 {
            self.verify_non_genesis(block)?;
        } else if block.prev_block_hash != vec![0; 32] {
            return Err(BlockError::InvalidGenesisFormat);
        }
        Ok(())
    }


    fn verify_non_genesis(&self, block: &Block) -> Result<(), BlockError> {
        let prev = &self.blocks[block.index - 1];
        if block.timestamp <= prev.timestamp {
            return Err(BlockError::AChronologicalTimestamp);
        }
        if block.prev_block_hash != prev.hash {
            return Err(BlockError::MismatchedPreviousHash);
        }
        Ok(())
    }

    fn process_transactions(&mut self, block: &Block) -> Result<(), BlockError> {
        let (coinbase, txs) = block.transactions.split_first().ok_or(BlockError::InvalidInput)?;
        if !coinbase.is_coinbase() {
            return Err(BlockError::InvalidCoinbase);
        }

        let mut spent = HashSet::new();
        let mut created = HashSet::new();
        let mut total_fee = 0;

        for tx in txs {
            let input_hashes = tx.input_hashes();
            if !input_hashes.is_subset(&self.unspent_outputs) || !input_hashes.is_disjoint(&spent) {
                return Err(BlockError::InvalidInput);
            }
            if tx.output_value() > tx.input_value() {
                return Err(BlockError::InsufficientInput);
            }
            total_fee += tx.input_value() - tx.output_value();
            spent.extend(input_hashes);
            created.extend(tx.output_hashes());
        }

        if coinbase.output_value() < total_fee {
            return Err(BlockError::InvalidCoinbase);
        }
        created.extend(coinbase.output_hashes());

        self.unspent_outputs.retain(|h| !spent.contains(h));
        self.unspent_outputs.extend(created);
        Ok(())
    }

    pub fn verify_all_blocks(&self) -> Result<(), BlockError> {
        for (i, block) in self.blocks.iter().enumerate() {
            self.verify_block(block, i)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn last_hash(&self) -> Vec<u8> {
        self.blocks.last().map(|b| b.hash.clone()).unwrap_or_else(|| vec![0; 32])
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}
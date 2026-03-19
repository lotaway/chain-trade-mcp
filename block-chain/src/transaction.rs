use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use crate::hashable::Hashable;
use crate::BlockHash;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub to: String,
    pub value: u64,
}

impl Hashable for Output {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = self.to.as_bytes().to_vec();
        bytes.extend(&self.value.to_le_bytes());
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<Output>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn input_value(&self) -> u64 {
        self.inputs.iter().map(|input| input.value).sum()
    }

    pub fn output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn input_hashes(&self) -> HashSet<BlockHash> {
        self.inputs.iter().map(|input| input.hash()).collect()
    }

    pub fn output_hashes(&self) -> HashSet<BlockHash> {
        self.outputs.iter().map(|output| output.hash()).collect()
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }
}

impl Hashable for Transaction {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for input in &self.inputs {
            bytes.extend(input.bytes());
        }
        for output in &self.outputs {
            bytes.extend(output.bytes());
        }
        bytes
    }
}

impl Hashable for Vec<Transaction> {
    fn bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|tx| tx.bytes()).collect()
    }
}
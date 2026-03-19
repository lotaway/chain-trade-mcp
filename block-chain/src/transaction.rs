use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use crate::hashable::Hashable;
use crate::lib::{Address, BlockHash, u64_bytes};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub to: Address,
    pub value: u64,
}

impl Hashable for Output {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.to.as_bytes());
        bytes.extend(&u64_bytes(&self.value));
        bytes
    }
}

trait IntVec {
    fn to_vec_u8(&self) -> Vec<u8>;
}

impl IntVec for Vec<Output> {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.iter().flat_map(|input| input.bytes()).collect::<Vec<u8>>()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<Output>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn input_val(&self) -> u64 {
        self.inputs.iter().map(|input| input.value).sum()
    }

    pub fn output_val(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn input_to_bytes(&self) -> Vec<u8> {
        self.inputs.to_vec_u8()
    }

    pub fn output_to_bytes(&self) -> Vec<u8> {
        self.outputs.to_vec_u8()
    }

    pub fn input_hashes(&self) -> HashSet<BlockHash> {
        self.inputs.iter().map(|input| input.hash()).collect::<HashSet<BlockHash>>()
    }

    pub fn output_hashes(&self) -> HashSet<BlockHash> {
        self.outputs.iter().map(|output| output.hash()).collect::<HashSet<BlockHash>>()
    }

    // todo!("The value of the coinbase transaction is not validated.");
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 0
    }
}

impl Hashable for Transaction {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = self.input_to_bytes();
        bytes.append(self.output_to_bytes().as_mut());
        bytes
    }
}

impl Hashable for Vec<Transaction> {
    fn bytes(&self) -> Vec<u8> {
        self.iter().flat_map(|transaction| transaction.bytes()).collect::<Vec<u8>>()
    }
}
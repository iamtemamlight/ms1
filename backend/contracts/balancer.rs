// ==============================================================================
// Balancer V2 Contract ABIs and Types
// ==============================================================================

use ethers_core::types::{Address, U256};

// Balancer V2 IVault ABI
pub const IVAULT_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "address", "name": "recipient", "type": "address"},
      {"internalType": "address[]", "name": "tokens", "type": "address[]"},
      {"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"},
      {"internalType": "bytes", "name": "userData", "type": "bytes"}
    ],
    "name": "flashLoan",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {"internalType": "address", "name": "recipient", "type": "address"},
      {"internalType": "address[]", "name": "tokens", "type": "address[]"},
      {"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"}
    ],
    "name": "sendWrapped",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  }
]
"#;

pub const IFLASH_LOAN_REPAYER_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "address", "name": "token", "type": "address"},
      {"internalType": "uint256", "name": "amount", "type": "uint256"}
    ],
    "name": "repayToken",
    "outputs": [],
    "stateMutability": "payable",
    "type": "function"
  }
]
"#;

/// Balancer V2 Vault address on Ethereum mainnet
pub const BALANCER_VAULT: &str = "0xBA12222222228d8Ba445958a75a0704d566BF2C8";

#[derive(Debug, Clone)]
pub struct BalancerFlashLoan {
    pub recipient: Address,
    pub tokens: Vec<Address>,
    pub amounts: Vec<U256>,
    pub user_data: Vec<u8>,
}

impl BalancerFlashLoan {
    pub fn new(recipient: Address, tokens: Vec<Address>, amounts: Vec<U256>) -> Self {
        Self {
            recipient,
            tokens,
            amounts,
            user_data: vec![],
        }
    }
}
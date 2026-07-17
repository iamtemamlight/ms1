// ==============================================================================
// dYdX Contract ABIs and Types
// ==============================================================================

use ethers_core::types::{Address, U256};

// dYdX SoloMargin ABI (simplified)
pub const ISOLO_MARGIN_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "bytes", "name": "data", "type": "bytes"}
    ],
    "name": "operate",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {"internalType": "uint256", "name": "accountId", "type": "uint256"}
    ],
    "name": "getAccountInfo",
    "outputs": [
      {"internalType": "address", "name": "owner", "type": "address"},
      {"internalType": "uint256", "name": "number", "type": "uint256"}
    ],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

pub const IEXCHANGE_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "bytes", "name": "data", "type": "bytes"}
    ],
    "name": "getTradeInfo",
    "outputs": [
      {"internalType": "uint256", "name": "orderId", "type": "uint256"},
      {"internalType": "address", "name": "maker", "type": "address"},
      {"internalType": "uint256", "name": "price", "type": "uint256"},
      {"internalType": "uint256", "name": "amount", "type": "uint256"}
    ],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

/// dYdX SoloMargin address on Ethereum mainnet
pub const DYDX_SOLO_MARGIN: &str = "0x1E0447e19a294b70e95920aD397f145214B6B76C";

#[derive(Debug, Clone)]
pub struct SoloMarginOperation {
    pub action_type: u32,
    pub account_id: u32,
    pub amount: U256,
    pub asset: Address,
    pub premium: U256,
}

impl SoloMarginOperation {
    pub fn new_deposit(account_id: u32, asset: Address, amount: U256) -> Self {
        Self {
            action_type: 0, // Deposit
            account_id,
            amount,
            asset,
            premium: U256::zero(),
        }
    }

    pub fn new_withdraw(account_id: u32, asset: Address, amount: U256) -> Self {
        Self {
            action_type: 1, // Withdraw
            account_id,
            amount,
            asset,
            premium: U256::zero(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&self.action_type.to_be_bytes());
        encoded.extend_from_slice(&self.account_id.to_be_bytes());
        let mut amount_bytes = [0u8; 32];
        self.amount.to_big_endian(&mut amount_bytes);
        encoded.extend_from_slice(&amount_bytes);
        encoded.extend_from_slice(&self.asset.as_bytes());
        encoded
    }
}
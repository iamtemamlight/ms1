// ==============================================================================
// Aave V3 Contract ABIs and Types
// ==============================================================================

use ethers_core::types::{Address, U256};

// Aave V3 IPool ABI (simplified for flash loans)
// Full ABI would be much longer - this is the critical subset
pub const IPOOL_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "address", "name": "receiverAddress", "type": "address"},
      {"internalType": "address[]", "name": "assets", "type": "address[]"},
      {"internalType": "uint256[]", "name": "amounts", "type": "uint256[]"},
      {"internalType": "uint256[]", "name": "modes", "type": "uint256[]"},
      {"internalType": "address", "name": "onBehalfOf", "type": "address"},
      {"internalType": "bytes", "name": "params", "type": "bytes"},
      {"internalType": "uint16", "name": "referralCode", "type": "uint16"}
    ],
    "name": "flashLoanSimple",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "ADDRESSES_PROVIDER",
    "outputs": [{"internalType": "contract IPoolAddressesProvider", "name": "", "type": "address"}],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {"internalType": "address", "name": "asset", "type": "address"},
      {"internalType": "uint256", "name": "amount", "type": "uint256"}
    ],
    "name": "getReserveData",
    "outputs": [
      {"internalType": "uint256", "name": "availableLiquidity", "type": "uint256"},
      {"internalType": "uint256", "name": "totalStableDebt", "type": "uint256"},
      {"internalType": "uint256", "name": "totalVariableDebt", "type": "uint256"},
      {"internalType": "uint256", "name": "liquidityRate", "type": "uint256"},
      {"internalType": "uint256", "name": "stableBorrowRate", "type": "uint256"},
      {"internalType": "uint256", "name": "variableBorrowRate", "type": "uint256"},
      {"internalType": "uint256", "name": "liquidityIndex", "type": "uint256"},
      {"internalType": "uint256", "name": "variableBorrowIndex", "type": "uint256"},
      {"internalType": "uint128", "name": "lastUpdateTimestamp", "type": "uint128"}
    ],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

pub const IPOOL_ADDRESSES_PROVIDER_ABI: &str = r#"
[
  {
    "inputs": [],
    "name": "getPool",
    "outputs": [{"internalType": "address", "name": "pool", "type": "address"}],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

pub const FLASHLOAN_SIMPLE_RECEIVER_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "address", "name": "initiator", "type": "address"},
      {"internalType": "address", "name": "asset", "type": "address"},
      {"internalType": "uint256", "name": "amount", "type": "uint256"},
      {"internalType": "uint256", "name": "premium", "type": "uint256"},
      {"internalType": "bytes", "name": "params", "type": "bytes"}
    ],
    "name": "executeOperation",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "ADDRESSES_PROVIDER",
    "outputs": [{"internalType": "contract IPoolAddressesProvider", "name": "", "type": "address"}],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

/// Common Aave V3 pool addresses
pub struct AavePoolAddresses;

impl AavePoolAddresses {
    pub const ETHEREUM: &'static str = "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2";
    pub const ARBITRUM: &'static str = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
    pub const OPTIMISM: &'static str = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
    pub const POLYGON: &'static str = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
    pub const AVALANCHE: &'static str = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
    pub const BASE: &'static str = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
}

/// Flash loan premium tier (basis points)
/// 0 = Stablecoins, 10000 = 1% fee
pub struct FlashLoanPremiums;

impl FlashLoanPremiums {
    pub const STABLE: u64 = 5;        // 0.05%
    pub const VOLATILE: u64 = 10;     // 0.10%
    pub const SPECIAL: u64 = 0;       // 0.00% for governance tokens
    pub const TX_FEE: u64 = 0;        // Aave fees are currently 0
}

#[derive(Debug, Clone)]
pub struct FlashLoanRequest {
    pub receiver_address: Address,
    pub assets: Vec<Address>,
    pub amounts: Vec<U256>,
    pub modes: Vec<u32>,      // 0 = no debt, 2 = stable, 1 = variable
    pub on_behalf_of: Address,
    pub params: Vec<u8>,
    pub referral_code: u16,
}

impl FlashLoanRequest {
    pub fn new_simple(
        receiver: Address,
        asset: Address,
        amount: U256,
    ) -> Self {
        Self {
            receiver_address: receiver,
            assets: vec![asset],
            amounts: vec![amount],
            modes: vec![0],
            on_behalf_of: receiver,
            params: vec![],
            referral_code: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReserveData {
    pub available_liquidity: U256,
    pub total_stable_debt: U256,
    pub total_variable_debt: U256,
    pub liquidity_rate: U256,
    pub stable_borrow_rate: U256,
    pub variable_borrow_rate: U256,
    pub liquidity_index: U256,
    pub variable_borrow_index: U256,
    pub last_update_timestamp: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_loan_request_creation() {
        let receiver = Address::zero();
        let asset = Address::zero();
        let amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH

        let req = FlashLoanRequest::new_simple(receiver, asset, amount);
        assert_eq!(req.assets.len(), 1);
        assert_eq!(req.amounts[0], amount);
        assert_eq!(req.modes[0], 0);
    }
}
// ==============================================================================
// Uniswap V3 Contract ABIs and Types
// ==============================================================================

use ethers_core::types::{Address, U256, Bytes};

// Uniswap V3 ISwapRouter ABI
pub const ISWAP_ROUTER_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "bytes", "name": "path", "type": "bytes"},
      {"internalType": "address", "name": "recipient", "type": "address"},
      {"internalType": "uint256", "name": "amountOut", "type": "uint256"},
      {"internalType": "uint256", "name": "amountInMax", "type": "uint256"},
      {"internalType": "uint160", "name": "sqrtPriceLimitX96", "type": "uint160"}
    ],
    "name": "exactOutput",
    "outputs": [{"internalType": "uint256", "name": "amountIn", "type": "uint256"}],
    "stateMutability": "payable",
    "type": "function"
  },
  {
    "inputs": [
      {"internalType": "bytes", "name": "path", "type": "bytes"},
      {"internalType": "address", "name": "recipient", "type": "address"},
      {"internalType": "uint256", "name": "amountIn", "type": "uint256"},
      {"internalType": "uint256", "name": "amountOutMinimum", "type": "uint256"},
      {"internalType": "uint160", "name": "sqrtPriceLimitX96", "type": "uint160"}
    ],
    "name": "exactInput",
    "outputs": [{"internalType": "uint256", "name": "amountOut", "type": "uint256"}],
    "stateMutability": "payable",
    "type": "function"
  }
]
"#;

pub const IQOUTER_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "bytes", "name": "path", "type": "bytes"},
      {"internalType": "uint256", "name": "amountIn", "type": "uint256"}
    ],
    "name": "quoteExactInput",
    "outputs": [{"internalType": "uint256", "name": "amountOut", "type": "uint256"}],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {"internalType": "bytes", "name": "path", "type": "bytes"},
      {"internalType": "uint256", "name": "amountOut", "type": "uint256"}
    ],
    "name": "quoteExactOutput",
    "outputs": [{"internalType": "uint256", "name": "amountIn", "type": "uint256"}],
    "stateMutability": "view",
    "type": "function"
  }
]
"#;

pub const INONFUNGIBLE_POSITION_MANAGER_ABI: &str = r#"
[
  {
    "inputs": [
      {"internalType": "address", "name": "tokenA", "type": "address"},
      {"internalType": "address", "name": "tokenB", "type": "address"},
      {"internalType": "uint24", "name": "fee", "type": "uint24"},
      {"internalType": "int24", "name": "tickLower", "type": "int24"},
      {"internalType": "int24", "name": "tickUpper", "type": "int24"},
      {"internalType": "uint256", "name": "amount0Desired", "type": "uint256"},
      {"internalType": "uint256", "name": "amount1Desired", "type": "uint256"},
      {"internalType": "uint256", "name": "amount0Min", "type": "uint256"},
      {"internalType": "uint256", "name": "amount1Min", "type": "uint256"},
      {"internalType": "address", "name": "recipient", "type": "address"},
      {"internalType": "uint256", "name": "deadline", "type": "uint256"}
    ],
    "name": "mint",
    "outputs": [
      {"internalType": "uint256", "name": "tokenId", "type": "uint256"},
      {"internalType": "bytes", "name": "data", "type": "bytes"}
    ],
    "stateMutability": "payable",
    "type": "function"
  }
]
"#;

/// Uniswap V3 router addresses per chain
pub struct UniswapAddresses;

impl UniswapAddresses {
    pub const V3_ROUTER_ETH: &'static str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
    pub const V3_ROUTER_ARBITRUM: &'static str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
    pub const V3_ROUTER_OPTIMISM: &'static str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
    pub const V3_QUOTER_ETH: &'static str = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6";
    pub const V3_QUOTER_V2: &'static str = "0x61fFE014bA17989E743c5F6cB21bF9697530B21e";
}

/// Uniswap V3 fee tiers (in hundredths of a bip)
pub struct UniswapFeeTiers;

impl UniswapFeeTiers {
    pub const LOW: u32 = 500;      // 0.05%
    pub const MEDIUM: u32 = 3000;  // 0.30%
    pub const HIGH: u32 = 10000;   // 1.00%
}

#[derive(Debug, Clone)]
pub struct SwapPath {
    pub token_in: Address,
    pub token_out: Address,
    pub fee: u32,
    pub sqrt_price_limit_x96: U256,
}

#[derive(Debug, Clone)]
pub struct SwapParameters {
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub recipient: Address,
    pub deadline: U256,
}

impl SwapParameters {
    pub fn encode_exact_input(&self, path: &SwapPath) -> Bytes {
        // Encode exactInputSwap calldata
        // In production, use ethers::contract::AbiEncode
        let mut encoded = Vec::new();
        // Function selector: exactInput((bytes,address,uint256,uint256,uint160))
        encoded.extend_from_slice(&[0xf2, 0x8a, 0xe3, 0x8f]); // truncated
        // Path encoding (contains fee)
        // ... implementation details
        Bytes::from(encoded)
    }
}
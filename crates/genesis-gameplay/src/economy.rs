//! Economy system.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Economy error types.
#[derive(Debug, Error)]
pub enum EconomyError {
    /// Insufficient funds
    #[error("Insufficient funds: need {needed}, have {have}")]
    InsufficientFunds {
        /// Amount needed
        needed: u64,
        /// Amount available
        have: u64,
    },
    /// Invalid price
    #[error("Invalid price")]
    InvalidPrice,
}

/// Result type for economy operations.
pub type EconomyResult<T> = Result<T, EconomyError>;

/// A wallet holding currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Current balance
    balance: u64,
}

impl Wallet {
    /// Creates a new wallet with initial balance.
    #[must_use]
    pub const fn new(initial: u64) -> Self {
        Self { balance: initial }
    }

    /// Returns the current balance.
    #[must_use]
    pub const fn balance(&self) -> u64 {
        self.balance
    }

    /// Adds currency to the wallet.
    pub fn earn(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount);
    }

    /// Removes currency from the wallet.
    pub fn spend(&mut self, amount: u64) -> EconomyResult<()> {
        if self.balance < amount {
            return Err(EconomyError::InsufficientFunds {
                needed: amount,
                have: self.balance,
            });
        }
        self.balance -= amount;
        Ok(())
    }

    /// Transfers currency to another wallet.
    pub fn transfer(&mut self, to: &mut Wallet, amount: u64) -> EconomyResult<()> {
        self.spend(amount)?;
        to.earn(amount);
        Ok(())
    }
}

/// Item price information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPrice {
    /// Base buy price
    pub base_buy: u64,
    /// Base sell price
    pub base_sell: u64,
    /// Price volatility (0-100)
    pub volatility: u8,
}

impl ItemPrice {
    /// Creates a new price.
    #[must_use]
    pub const fn new(base_buy: u64, base_sell: u64) -> Self {
        Self {
            base_buy,
            base_sell,
            volatility: 10,
        }
    }

    /// Calculates buy price with supply modifier.
    #[must_use]
    pub fn buy_price(&self, supply_modifier: f32) -> u64 {
        let modifier = 1.0 + (1.0 - supply_modifier) * (self.volatility as f32 / 100.0);
        (self.base_buy as f32 * modifier) as u64
    }

    /// Calculates sell price with supply modifier.
    #[must_use]
    pub fn sell_price(&self, supply_modifier: f32) -> u64 {
        let modifier = 1.0 + (supply_modifier - 1.0) * (self.volatility as f32 / 100.0);
        (self.base_sell as f32 * modifier) as u64
    }
}

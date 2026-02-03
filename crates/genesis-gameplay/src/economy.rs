//! Economy system with prices, transactions, and multiple currencies.

use genesis_common::ItemTypeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Item not found in registry
    #[error("Item not found in price registry: {0:?}")]
    ItemNotFound(ItemTypeId),
    /// Invalid quantity
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(u32),
}

/// Result type for economy operations.
pub type EconomyResult<T> = Result<T, EconomyError>;

/// Currency type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyType {
    /// Gold (primary currency)
    Gold,
    /// Silver (secondary currency)
    Silver,
    /// Reputation tokens
    ReputationToken,
    /// Custom currency
    Custom(u32),
}

impl Default for CurrencyType {
    fn default() -> Self {
        Self::Gold
    }
}

/// A wallet holding multiple currencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Currency balances
    balances: HashMap<CurrencyType, u64>,
}

impl Wallet {
    /// Creates a new empty wallet.
    #[must_use]
    pub fn new(initial_gold: u64) -> Self {
        let mut balances = HashMap::new();
        if initial_gold > 0 {
            balances.insert(CurrencyType::Gold, initial_gold);
        }
        Self { balances }
    }

    /// Creates an empty wallet with no currency.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    /// Returns the balance of a specific currency.
    #[must_use]
    pub fn balance(&self) -> u64 {
        self.balance_of(CurrencyType::Gold)
    }

    /// Returns the balance of a specific currency type.
    #[must_use]
    pub fn balance_of(&self, currency: CurrencyType) -> u64 {
        self.balances.get(&currency).copied().unwrap_or(0)
    }

    /// Adds currency to the wallet.
    pub fn earn(&mut self, amount: u64) {
        self.earn_currency(CurrencyType::Gold, amount);
    }

    /// Adds a specific currency type to the wallet.
    pub fn earn_currency(&mut self, currency: CurrencyType, amount: u64) {
        let current = self.balance_of(currency);
        self.balances
            .insert(currency, current.saturating_add(amount));
    }

    /// Removes currency from the wallet.
    pub fn spend(&mut self, amount: u64) -> EconomyResult<()> {
        self.spend_currency(CurrencyType::Gold, amount)
    }

    /// Removes a specific currency type from the wallet.
    pub fn spend_currency(&mut self, currency: CurrencyType, amount: u64) -> EconomyResult<()> {
        let current = self.balance_of(currency);
        if current < amount {
            return Err(EconomyError::InsufficientFunds {
                needed: amount,
                have: current,
            });
        }
        self.balances.insert(currency, current - amount);
        Ok(())
    }

    /// Transfers currency to another wallet.
    pub fn transfer(&mut self, to: &mut Wallet, amount: u64) -> EconomyResult<()> {
        self.transfer_currency(to, CurrencyType::Gold, amount)
    }

    /// Transfers a specific currency type to another wallet.
    pub fn transfer_currency(
        &mut self,
        to: &mut Wallet,
        currency: CurrencyType,
        amount: u64,
    ) -> EconomyResult<()> {
        self.spend_currency(currency, amount)?;
        to.earn_currency(currency, amount);
        Ok(())
    }

    /// Returns all currency types in the wallet.
    pub fn currencies(&self) -> impl Iterator<Item = (CurrencyType, u64)> + '_ {
        self.balances.iter().map(|(&c, &v)| (c, v))
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
    /// Currency type for this item
    pub currency: CurrencyType,
}

impl ItemPrice {
    /// Creates a new price in gold.
    #[must_use]
    pub const fn new(base_buy: u64, base_sell: u64) -> Self {
        Self {
            base_buy,
            base_sell,
            volatility: 10,
            currency: CurrencyType::Gold,
        }
    }

    /// Creates a price with custom currency.
    #[must_use]
    pub const fn with_currency(base_buy: u64, base_sell: u64, currency: CurrencyType) -> Self {
        Self {
            base_buy,
            base_sell,
            volatility: 10,
            currency,
        }
    }

    /// Sets the volatility.
    #[must_use]
    pub const fn with_volatility(mut self, volatility: u8) -> Self {
        self.volatility = volatility;
        self
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

/// A record of a completed transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Item type involved
    pub item: ItemTypeId,
    /// Quantity bought or sold
    pub quantity: u32,
    /// Total price paid/received
    pub total_price: u64,
    /// Currency used
    pub currency: CurrencyType,
    /// Whether this was a purchase (true) or sale (false)
    pub is_purchase: bool,
    /// Timestamp (game tick)
    pub timestamp: u64,
}

/// Registry of item prices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriceRegistry {
    /// Prices by item type
    prices: HashMap<ItemTypeId, ItemPrice>,
}

impl PriceRegistry {
    /// Creates a new empty price registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a price for an item.
    pub fn register(&mut self, item: ItemTypeId, price: ItemPrice) {
        self.prices.insert(item, price);
    }

    /// Gets the price for an item.
    #[must_use]
    pub fn get(&self, item: ItemTypeId) -> Option<&ItemPrice> {
        self.prices.get(&item)
    }

    /// Checks if an item has a registered price.
    #[must_use]
    pub fn has_price(&self, item: ItemTypeId) -> bool {
        self.prices.contains_key(&item)
    }

    /// Returns all registered items.
    pub fn items(&self) -> impl Iterator<Item = ItemTypeId> + '_ {
        self.prices.keys().copied()
    }

    /// Returns the number of registered prices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.prices.len()
    }

    /// Checks if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.prices.is_empty()
    }
}

/// Economy manager handling transactions.
#[derive(Debug, Default)]
pub struct Economy {
    /// Price registry
    prices: PriceRegistry,
    /// Transaction history
    history: Vec<Transaction>,
    /// Current game tick for timestamps
    current_tick: u64,
    /// Global supply modifier (affects all prices)
    global_supply_modifier: f32,
}

impl Economy {
    /// Creates a new economy system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prices: PriceRegistry::new(),
            history: Vec::new(),
            current_tick: 0,
            global_supply_modifier: 1.0,
        }
    }

    /// Creates an economy with an existing price registry.
    #[must_use]
    pub fn with_prices(prices: PriceRegistry) -> Self {
        Self {
            prices,
            history: Vec::new(),
            current_tick: 0,
            global_supply_modifier: 1.0,
        }
    }

    /// Returns a reference to the price registry.
    #[must_use]
    pub fn prices(&self) -> &PriceRegistry {
        &self.prices
    }

    /// Returns a mutable reference to the price registry.
    pub fn prices_mut(&mut self) -> &mut PriceRegistry {
        &mut self.prices
    }

    /// Advances the game tick.
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    /// Sets the global supply modifier.
    pub fn set_supply_modifier(&mut self, modifier: f32) {
        self.global_supply_modifier = modifier.max(0.1);
    }

    /// Calculates the buy price for an item.
    pub fn calculate_buy_price(&self, item: ItemTypeId, quantity: u32) -> EconomyResult<u64> {
        let price = self
            .prices
            .get(item)
            .ok_or(EconomyError::ItemNotFound(item))?;
        let unit_price = price.buy_price(self.global_supply_modifier);
        Ok(unit_price * quantity as u64)
    }

    /// Calculates the sell price for an item.
    pub fn calculate_sell_price(&self, item: ItemTypeId, quantity: u32) -> EconomyResult<u64> {
        let price = self
            .prices
            .get(item)
            .ok_or(EconomyError::ItemNotFound(item))?;
        let unit_price = price.sell_price(self.global_supply_modifier);
        Ok(unit_price * quantity as u64)
    }

    /// Executes a purchase (player buys from merchant).
    pub fn buy(
        &mut self,
        wallet: &mut Wallet,
        item: ItemTypeId,
        quantity: u32,
    ) -> EconomyResult<u64> {
        if quantity == 0 {
            return Err(EconomyError::InvalidQuantity(quantity));
        }

        let price_info = self
            .prices
            .get(item)
            .ok_or(EconomyError::ItemNotFound(item))?;
        let currency = price_info.currency;
        let total_price = self.calculate_buy_price(item, quantity)?;

        wallet.spend_currency(currency, total_price)?;

        self.history.push(Transaction {
            item,
            quantity,
            total_price,
            currency,
            is_purchase: true,
            timestamp: self.current_tick,
        });

        Ok(total_price)
    }

    /// Executes a sale (player sells to merchant).
    pub fn sell(
        &mut self,
        wallet: &mut Wallet,
        item: ItemTypeId,
        quantity: u32,
    ) -> EconomyResult<u64> {
        if quantity == 0 {
            return Err(EconomyError::InvalidQuantity(quantity));
        }

        let price_info = self
            .prices
            .get(item)
            .ok_or(EconomyError::ItemNotFound(item))?;
        let currency = price_info.currency;
        let total_price = self.calculate_sell_price(item, quantity)?;

        wallet.earn_currency(currency, total_price);

        self.history.push(Transaction {
            item,
            quantity,
            total_price,
            currency,
            is_purchase: false,
            timestamp: self.current_tick,
        });

        Ok(total_price)
    }

    /// Returns transaction history.
    pub fn history(&self) -> &[Transaction] {
        &self.history
    }

    /// Returns recent transactions (last N).
    pub fn recent_transactions(&self, count: usize) -> &[Transaction] {
        let start = self.history.len().saturating_sub(count);
        &self.history[start..]
    }

    /// Clears transaction history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_new() {
        let wallet = Wallet::new(1000);
        assert_eq!(wallet.balance(), 1000);
    }

    #[test]
    fn test_wallet_empty() {
        let wallet = Wallet::empty();
        assert_eq!(wallet.balance(), 0);
    }

    #[test]
    fn test_wallet_earn_spend() {
        let mut wallet = Wallet::new(100);
        wallet.earn(50);
        assert_eq!(wallet.balance(), 150);

        assert!(wallet.spend(100).is_ok());
        assert_eq!(wallet.balance(), 50);
    }

    #[test]
    fn test_wallet_insufficient_funds() {
        let mut wallet = Wallet::new(50);
        let result = wallet.spend(100);
        assert!(matches!(
            result,
            Err(EconomyError::InsufficientFunds {
                needed: 100,
                have: 50
            })
        ));
    }

    #[test]
    fn test_wallet_multiple_currencies() {
        let mut wallet = Wallet::new(100);
        wallet.earn_currency(CurrencyType::Silver, 500);
        wallet.earn_currency(CurrencyType::ReputationToken, 10);

        assert_eq!(wallet.balance_of(CurrencyType::Gold), 100);
        assert_eq!(wallet.balance_of(CurrencyType::Silver), 500);
        assert_eq!(wallet.balance_of(CurrencyType::ReputationToken), 10);
    }

    #[test]
    fn test_wallet_transfer() {
        let mut from = Wallet::new(1000);
        let mut to = Wallet::empty();

        assert!(from.transfer(&mut to, 300).is_ok());
        assert_eq!(from.balance(), 700);
        assert_eq!(to.balance(), 300);
    }

    #[test]
    fn test_price_registry() {
        let mut registry = PriceRegistry::new();
        let item = ItemTypeId::new(1);

        registry.register(item, ItemPrice::new(100, 50));
        assert!(registry.has_price(item));
        assert_eq!(registry.len(), 1);

        let price = registry.get(item).expect("should exist");
        assert_eq!(price.base_buy, 100);
        assert_eq!(price.base_sell, 50);
    }

    #[test]
    fn test_economy_buy() {
        let mut economy = Economy::new();
        economy
            .prices_mut()
            .register(ItemTypeId::new(1), ItemPrice::new(100, 50));

        let mut wallet = Wallet::new(1000);
        let spent = economy
            .buy(&mut wallet, ItemTypeId::new(1), 3)
            .expect("should succeed");

        assert_eq!(spent, 300);
        assert_eq!(wallet.balance(), 700);
        assert_eq!(economy.history().len(), 1);
    }

    #[test]
    fn test_economy_sell() {
        let mut economy = Economy::new();
        economy
            .prices_mut()
            .register(ItemTypeId::new(1), ItemPrice::new(100, 50));

        let mut wallet = Wallet::new(0);
        let earned = economy
            .sell(&mut wallet, ItemTypeId::new(1), 2)
            .expect("should succeed");

        assert_eq!(earned, 100);
        assert_eq!(wallet.balance(), 100);
    }

    #[test]
    fn test_economy_item_not_found() {
        let economy = Economy::new();
        let mut wallet = Wallet::new(1000);

        let result = economy.calculate_buy_price(ItemTypeId::new(999), 1);
        assert!(matches!(result, Err(EconomyError::ItemNotFound(_))));

        // Create a mutable economy for the buy test
        let mut economy = Economy::new();
        let result = economy.buy(&mut wallet, ItemTypeId::new(999), 1);
        assert!(matches!(result, Err(EconomyError::ItemNotFound(_))));
    }

    #[test]
    fn test_economy_invalid_quantity() {
        let mut economy = Economy::new();
        economy
            .prices_mut()
            .register(ItemTypeId::new(1), ItemPrice::new(100, 50));

        let mut wallet = Wallet::new(1000);
        let result = economy.buy(&mut wallet, ItemTypeId::new(1), 0);
        assert!(matches!(result, Err(EconomyError::InvalidQuantity(0))));
    }

    #[test]
    fn test_economy_transaction_history() {
        let mut economy = Economy::new();
        economy
            .prices_mut()
            .register(ItemTypeId::new(1), ItemPrice::new(100, 50));

        let mut wallet = Wallet::new(1000);
        let _ = economy.buy(&mut wallet, ItemTypeId::new(1), 1);
        let _ = economy.sell(&mut wallet, ItemTypeId::new(1), 1);
        let _ = economy.buy(&mut wallet, ItemTypeId::new(1), 2);

        assert_eq!(economy.history().len(), 3);

        let recent = economy.recent_transactions(2);
        assert_eq!(recent.len(), 2);
        assert!(!recent[0].is_purchase); // sell
        assert!(recent[1].is_purchase); // buy
    }

    #[test]
    fn test_price_volatility() {
        let price = ItemPrice::new(100, 50).with_volatility(50);

        // High supply = lower buy price, higher sell price
        let buy_high_supply = price.buy_price(1.5);
        let buy_low_supply = price.buy_price(0.5);
        assert!(buy_high_supply < buy_low_supply);

        let sell_high_supply = price.sell_price(1.5);
        let sell_low_supply = price.sell_price(0.5);
        assert!(sell_high_supply > sell_low_supply);
    }

    #[test]
    fn test_item_price_custom_currency() {
        let price = ItemPrice::with_currency(100, 50, CurrencyType::Silver);
        assert_eq!(price.currency, CurrencyType::Silver);
    }
}

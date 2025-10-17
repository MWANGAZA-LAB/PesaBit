/// Shared types used across all PesaBit services
/// 
/// This library defines common data types that represent core business concepts.
/// By centralizing these types, we ensure consistency across all services and avoid duplication.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a user in the system
/// This is used consistently across all services to identify users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an amount of money in Kenyan Shillings
/// Uses Decimal for precise financial calculations (no floating point errors)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct KesAmount(pub Decimal);

impl KesAmount {
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }
    
    pub fn from_major(major: i64) -> Self {
        Self(Decimal::new(major, 2)) // 2 decimal places for cents
    }
    
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }
    
    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }
}

/// Represents an amount in Bitcoin satoshis (smallest Bitcoin unit)
/// 1 Bitcoin = 100,000,000 satoshis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct SatAmount(pub i64);

impl SatAmount {
    pub fn new(sats: i64) -> Self {
        Self(sats)
    }
    
    pub fn zero() -> Self {
        Self(0)
    }
    
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }
}

/// Phone number in international E.164 format
/// Example: +254712345678 (Kenyan mobile number)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct PhoneNumber(pub String);

impl PhoneNumber {
    pub fn new(number: String) -> Result<Self, String> {
        // Basic validation - should start with + and be 10-15 digits
        if !number.starts_with('+') || number.len() < 10 || number.len() > 16 {
            return Err("Invalid phone number format".to_string());
        }
        Ok(Self(number))
    }
}

/// Lightning Network payment address (like email)
/// Example: john@pesa.co.ke
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct LightningAddress(pub String);

impl LightningAddress {
    pub fn new(username: &str, domain: &str) -> Self {
        Self(format!("{}@{}", username, domain))
    }
}

/// M-Pesa transaction reference code
/// Example: QL12XYZ789
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]  
pub struct MpesaCode(pub String);

/// Lightning Network invoice (BOLT11 format)
/// Used to request payments over Lightning
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct LightningInvoice(pub String);

/// Lightning payment preimage - proof that payment was completed
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct PaymentPreimage(pub String);

/// Different types of transactions in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    /// User deposits KES via M-Pesa, gets Bitcoin satoshis
    DepositMpesa,
    /// User withdraws Bitcoin to M-Pesa as KES
    WithdrawalMpesa, 
    /// User sends Lightning payment to someone else
    LightningSend,
    /// User receives Lightning payment from someone else
    LightningReceive,
}

/// Current status of a transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Transaction created but not yet processed
    Pending,
    /// Currently being processed (M-Pesa confirmation, Lightning routing)
    Processing,
    /// Successfully completed
    Completed,
    /// Failed due to error (insufficient funds, timeout, etc.)
    Failed,
    /// Failed transaction that was refunded to user
    Refunded,
}

/// User's KYC (Know Your Customer) verification status
/// Higher tiers allow larger transaction limits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "kyc_status", rename_all = "snake_case")]
pub enum KycStatus {
    /// No verification done yet
    None,
    /// Documents submitted, waiting for review  
    Pending,
    /// Identity verified successfully
    Verified,
    /// Verification failed or rejected
    Rejected,
}

/// KYC verification tier determining transaction limits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "kyc_tier", rename_all = "snake_case")]  
pub enum KycTier {
    /// Phone verification only - 10,000 KES/day limit
    Tier0,
    /// ID verified - 100,000 KES/day limit  
    Tier1,
    /// Full verification - unlimited
    Tier2,
}

/// Complete transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: UserId,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub amount_kes: Option<KesAmount>,
    pub amount_sats: Option<SatAmount>,
    pub exchange_rate: Option<Decimal>, // KES per 100k sats at time of transaction
    pub fee_kes: Option<KesAmount>,
    pub fee_sats: Option<SatAmount>,
    pub mpesa_code: Option<MpesaCode>,
    pub lightning_invoice: Option<LightningInvoice>,
    pub lightning_preimage: Option<PaymentPreimage>,
    pub metadata: serde_json::Value, // Flexible JSON for notes, recipient info, etc.
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// User's wallet balances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: UserId,
    /// Confirmed Bitcoin balance in satoshis
    pub balance_sats: SatAmount,
    /// Pending M-Pesa balance (before conversion to Bitcoin)
    pub balance_kes: KesAmount,
    /// Unconfirmed Lightning payments (pending confirmation)
    pub pending_balance_sats: SatAmount,
    pub updated_at: DateTime<Utc>,
}

/// Exchange rate between Bitcoin and Kenyan Shillings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub id: i32,
    /// Bitcoin price in KES (e.g., 5,300,000 KES per BTC)
    pub btc_kes: Decimal,
    /// Data source (blockchain.info, binance, etc.)
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2); // Should be unique
    }

    #[test] 
    fn test_phone_number_validation() {
        assert!(PhoneNumber::new("+254712345678".to_string()).is_ok());
        assert!(PhoneNumber::new("254712345678".to_string()).is_err()); // Missing +
        assert!(PhoneNumber::new("+123".to_string()).is_err()); // Too short
    }

    #[test]
    fn test_kes_amount() {
        let amount = KesAmount::from_major(1000); // 10.00 KES
        assert!(amount.is_positive());
        assert_eq!(amount.0.to_string(), "10.00");
    }
}
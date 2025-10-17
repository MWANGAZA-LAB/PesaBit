/// Payment domain models and business logic
/// 
/// This module defines the core payment operations, business rules, and validation
/// for M-Pesa deposits/withdrawals and Lightning Network transactions.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared_types::*;
use validator::Validate;

/// M-Pesa deposit request (user adds money via M-Pesa)
#[derive(Debug, Deserialize, Validate)]
pub struct MpesaDepositRequest {
    /// Amount in Kenyan Shillings to deposit
    #[validate(range(min = 10, max = 500000))] // Min 10 KES, Max 500k KES per transaction
    pub amount_kes: i32,
}

/// Response after initiating M-Pesa deposit
#[derive(Debug, Serialize)]
pub struct MpesaDepositResponse {
    pub transaction_id: String,
    pub checkout_request_id: String,
    pub amount_kes: KesAmount,
    pub estimated_sats: SatAmount,
    pub exchange_rate: Decimal,
    pub fee_kes: KesAmount,
    pub message: String,
}

/// M-Pesa withdrawal request (user cashes out Bitcoin)
#[derive(Debug, Deserialize, Validate)]
pub struct MpesaWithdrawalRequest {
    /// Amount in satoshis to withdraw
    #[validate(range(min = 1000))] // Minimum 1000 sats
    pub amount_sats: i64,
    /// Optional: specific phone number (defaults to user's registered number)
    pub recipient_phone: Option<String>,
}

/// Response after initiating M-Pesa withdrawal
#[derive(Debug, Serialize)]
pub struct MpesaWithdrawalResponse {
    pub transaction_id: String,
    pub amount_sats: SatAmount,
    pub amount_kes: KesAmount,
    pub exchange_rate: Decimal,
    pub fee_kes: KesAmount,
    pub fee_sats: SatAmount,
    pub recipient_phone: PhoneNumber,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
}

/// Lightning invoice creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceRequest {
    /// Amount in satoshis to request
    #[validate(range(min = 1, max = 100000000))] // 1 sat to 1 BTC
    pub amount_sats: i64,
    /// Optional description/memo for the payment
    #[validate(length(max = 500))]
    pub description: Option<String>,
    /// Invoice expiry in seconds (default: 1 hour)
    #[validate(range(min = 60, max = 86400))] // 1 minute to 24 hours
    pub expiry_seconds: Option<i32>,
}

/// Response after creating Lightning invoice
#[derive(Debug, Serialize)]
pub struct CreateInvoiceResponse {
    pub transaction_id: String,
    pub bolt11_invoice: LightningInvoice,
    pub payment_request: String, // Same as bolt11_invoice, for convenience
    pub amount_sats: SatAmount,
    pub description: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub qr_code_url: String, // URL to QR code image
}

/// Lightning payment request (user pays an invoice)
#[derive(Debug, Deserialize, Validate)]
pub struct PayInvoiceRequest {
    /// BOLT11 Lightning invoice to pay
    pub bolt11_invoice: String,
    /// Maximum fee willing to pay in satoshis (safety limit)
    #[validate(range(min = 0, max = 10000))]
    pub max_fee_sats: Option<i64>,
}

/// Response after attempting Lightning payment
#[derive(Debug, Serialize)]
pub struct PayInvoiceResponse {
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub amount_sats: SatAmount,
    pub fee_sats: SatAmount,
    pub payment_preimage: Option<PaymentPreimage>,
    pub failure_reason: Option<String>,
}

/// User's wallet balance information
#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub user_id: UserId,
    /// Confirmed Bitcoin balance (can be spent immediately)
    pub balance_sats: SatAmount,
    /// Equivalent value in KES (at current exchange rate)
    pub balance_kes_equivalent: KesAmount,
    /// Pending M-Pesa deposits (being converted to Bitcoin)
    pub pending_mpesa_kes: KesAmount,
    /// Unconfirmed Lightning payments (waiting for confirmation)
    pub pending_lightning_sats: SatAmount,
    /// Current exchange rate used for conversions
    pub exchange_rate: Decimal,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transaction history query parameters
#[derive(Debug, Deserialize)]
pub struct TransactionHistoryParams {
    /// Number of transactions to return (default: 20, max: 100)
    pub limit: Option<i32>,
    /// Offset for pagination (default: 0)
    pub offset: Option<i32>,
    /// Filter by transaction type
    pub transaction_type: Option<TransactionType>,
    /// Filter by status
    pub status: Option<TransactionStatus>,
    /// Filter transactions after this date
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter transactions before this date
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Transaction history response
#[derive(Debug, Serialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<TransactionSummary>,
    pub total_count: i64,
    pub has_more: bool,
}

/// Simplified transaction summary for history lists
#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub id: String,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub amount_kes: Option<KesAmount>,
    pub amount_sats: Option<SatAmount>,
    pub fee_kes: Option<KesAmount>,
    pub fee_sats: Option<SatAmount>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// M-Pesa callback from Safaricom (webhook payload)
#[derive(Debug, Deserialize)]
pub struct MpesaCallback {
    pub body: MpesaCallbackBody,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackBody {
    #[serde(rename = "stkCallback")]
    pub stk_callback: StkCallback,
}

#[derive(Debug, Deserialize)]
pub struct StkCallback {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "CallbackMetadata")]
    pub callback_metadata: Option<CallbackMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackMetadata {
    #[serde(rename = "Item")]
    pub item: Vec<CallbackItem>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackItem {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

/// Business rules and validation
impl MpesaDepositRequest {
    /// Calculate fees for M-Pesa deposit (1% fee)
    pub fn calculate_fee(&self) -> KesAmount {
        let fee = Decimal::from(self.amount_kes) * Decimal::new(1, 2); // 1%
        KesAmount::new(fee.max(Decimal::new(10, 0))) // Minimum 10 KES fee
    }

    /// Calculate net amount after fees
    pub fn net_amount(&self) -> KesAmount {
        let amount = Decimal::from(self.amount_kes);
        let fee = self.calculate_fee().0;
        KesAmount::new(amount - fee)
    }
}

impl MpesaWithdrawalRequest {
    /// Calculate fees for M-Pesa withdrawal
    pub fn calculate_fees(&self, exchange_rate: Decimal) -> (KesAmount, SatAmount) {
        // Convert sats to KES
        let kes_amount = self.sats_to_kes(exchange_rate);
        
        // Our fee: 1% of KES amount
        let our_fee_kes = kes_amount.0 * Decimal::new(1, 2); // 1%
        
        // M-Pesa fees (tiered based on amount)
        let mpesa_fee_kes = Self::calculate_mpesa_fee(kes_amount.0);
        
        let total_fee_kes = KesAmount::new(our_fee_kes + mpesa_fee_kes);
        let fee_sats = SatAmount::new(self.kes_to_sats(total_fee_kes.0, exchange_rate));
        
        (total_fee_kes, fee_sats)
    }

    /// Convert satoshis to KES at given exchange rate
    fn sats_to_kes(&self, btc_kes_rate: Decimal) -> KesAmount {
        let btc_amount = Decimal::from(self.amount_sats) / Decimal::new(100_000_000, 0); // sats to BTC
        KesAmount::new(btc_amount * btc_kes_rate)
    }

    /// Convert KES to satoshis at given exchange rate
    fn kes_to_sats(&self, kes_amount: Decimal, btc_kes_rate: Decimal) -> i64 {
        let btc_amount = kes_amount / btc_kes_rate;
        (btc_amount * Decimal::new(100_000_000, 0)).to_i64().unwrap_or(0)
    }

    /// Calculate M-Pesa withdrawal fees (Safaricom's tiered structure)
    fn calculate_mpesa_fee(amount: Decimal) -> Decimal {
        match amount.to_i64().unwrap_or(0) {
            1..=49 => Decimal::new(1, 0),           // 1 KES
            50..=100 => Decimal::new(5, 0),         // 5 KES  
            101..=500 => Decimal::new(7, 0),        // 7 KES
            501..=1000 => Decimal::new(13, 0),      // 13 KES
            1001..=1500 => Decimal::new(20, 0),     // 20 KES
            1501..=2500 => Decimal::new(25, 0),     // 25 KES
            2501..=3500 => Decimal::new(30, 0),     // 30 KES
            3501..=5000 => Decimal::new(35, 0),     // 35 KES
            5001..=7500 => Decimal::new(45, 0),     // 45 KES
            7501..=10000 => Decimal::new(55, 0),    // 55 KES
            10001..=15000 => Decimal::new(60, 0),   // 60 KES
            15001..=20000 => Decimal::new(65, 0),   // 65 KES
            20001..=25000 => Decimal::new(70, 0),   // 70 KES
            25001..=30000 => Decimal::new(75, 0),   // 75 KES
            _ => Decimal::new(105, 0),              // 105 KES for >30k
        }
    }
}

impl CreateInvoiceRequest {
    /// Get expiry duration (default: 1 hour)
    pub fn expiry_duration(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.expiry_seconds.unwrap_or(3600) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_fee_calculation() {
        let request = MpesaDepositRequest { amount_kes: 1000 };
        assert_eq!(request.calculate_fee().0, Decimal::new(10, 0)); // 1% = 10 KES
        assert_eq!(request.net_amount().0, Decimal::new(990, 0)); // 990 KES after fees
    }

    #[test]
    fn test_mpesa_fee_tiers() {
        assert_eq!(MpesaWithdrawalRequest::calculate_mpesa_fee(Decimal::new(50, 0)), Decimal::new(5, 0));
        assert_eq!(MpesaWithdrawalRequest::calculate_mpesa_fee(Decimal::new(1000, 0)), Decimal::new(13, 0));
        assert_eq!(MpesaWithdrawalRequest::calculate_mpesa_fee(Decimal::new(5000, 0)), Decimal::new(35, 0));
    }

    #[test]
    fn test_invoice_expiry() {
        let request = CreateInvoiceRequest {
            amount_sats: 1000,
            description: None,
            expiry_seconds: Some(1800), // 30 minutes
        };
        assert_eq!(request.expiry_duration(), chrono::Duration::seconds(1800));
        
        let default_request = CreateInvoiceRequest {
            amount_sats: 1000,
            description: None,
            expiry_seconds: None,
        };
        assert_eq!(default_request.expiry_duration(), chrono::Duration::seconds(3600)); // 1 hour default
    }
}
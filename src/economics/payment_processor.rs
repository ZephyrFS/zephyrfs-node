//! Multi-Currency Payment Processing Engine
//!
//! Handles payouts in multiple currencies for ZephyrFS volunteers

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

/// Multi-currency payment processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessor {
    /// Supported payment methods
    pub payment_methods: HashMap<PaymentMethod, PaymentMethodConfig>,
    /// Exchange rates for currency conversion
    pub exchange_rates: HashMap<Currency, f64>,
    /// Payment processing fees
    pub fee_structure: PaymentFeeStructure,
    /// Pending payments queue
    pub pending_payments: VecDeque<PaymentRequest>,
    /// Payment history
    pub payment_history: HashMap<String, Vec<PaymentRecord>>,
    /// Minimum payout thresholds
    pub min_payout_thresholds: HashMap<Currency, u64>,
    /// Processor configuration
    pub config: ProcessorConfig,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Cryptocurrency(CryptoNetwork),
    BankTransfer(BankTransferType),
    DigitalWallet(WalletProvider),
    StableCoin(StableCoinType),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CryptoNetwork {
    Bitcoin,
    Ethereum,
    Polygon,
    BinanceSmartChain,
    Solana,
    Cardano,
    ZephyrCoin, // Native token
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BankTransferType {
    ACH,        // US
    SEPA,       // Europe
    FasterPayments, // UK
    Interac,    // Canada
    PIX,        // Brazil
    UPI,        // India
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum WalletProvider {
    PayPal,
    Wise,
    Revolut,
    CashApp,
    Venmo,
    Zelle,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum StableCoinType {
    USDC,
    USDT,
    DAI,
    BUSD,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    CAD,
    AUD,
    BRL,
    INR,
    JPY,
    ZephyrCoin,
    Bitcoin,
    Ethereum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodConfig {
    pub enabled: bool,
    pub min_amount: u64,
    pub max_amount: u64,
    pub processing_time_hours: u32,
    pub supported_currencies: Vec<Currency>,
    pub geographic_restrictions: Vec<String>, // ISO country codes
    pub requires_kyc: bool,
    pub fee_structure: MethodFeeStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodFeeStructure {
    pub fixed_fee: u64,     // Fixed fee in cents/wei
    pub percentage_fee: f64, // Percentage fee (0.01 = 1%)
    pub network_fee: u64,   // Blockchain network fees
    pub exchange_fee: f64,  // Currency conversion fee
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFeeStructure {
    /// Base processing fee (0.5%)
    pub base_processing_fee: f64,
    /// Currency conversion fees
    pub conversion_fees: HashMap<Currency, f64>,
    /// Express processing fee (1%)
    pub express_fee: f64,
    /// KYC verification fee
    pub kyc_fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub request_id: String,
    pub volunteer_id: String,
    pub amount_tokens: u64,
    pub target_currency: Currency,
    pub payment_method: PaymentMethod,
    pub recipient_info: RecipientInfo,
    pub priority: PaymentPriority,
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientInfo {
    pub wallet_address: Option<String>,
    pub bank_account: Option<BankAccountInfo>,
    pub digital_wallet: Option<DigitalWalletInfo>,
    pub kyc_verified: bool,
    pub tax_info: Option<TaxInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountInfo {
    pub account_holder: String,
    pub account_number: String,
    pub routing_number: String,
    pub bank_name: String,
    pub swift_code: Option<String>,
    pub iban: Option<String>,
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalWalletInfo {
    pub provider: WalletProvider,
    pub wallet_id: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInfo {
    pub tax_id: String,
    pub tax_country: String,
    pub tax_exempt: bool,
    pub withholding_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentPriority {
    Standard,
    Express,
    Immediate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_id: String,
    pub request_id: String,
    pub volunteer_id: String,
    pub amount_tokens: u64,
    pub amount_paid: u64,
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub fees_paid: u64,
    pub exchange_rate: f64,
    pub processed_at: DateTime<Utc>,
    pub confirmation_hash: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    RequiresAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    pub batch_processing_enabled: bool,
    pub batch_size: usize,
    pub processing_interval_minutes: u32,
    pub max_retries: u32,
    pub retry_delay_minutes: u32,
    pub auto_currency_conversion: bool,
    pub fraud_detection_enabled: bool,
}

impl Default for PaymentFeeStructure {
    fn default() -> Self {
        let mut conversion_fees = HashMap::new();
        conversion_fees.insert(Currency::USD, 0.005);
        conversion_fees.insert(Currency::EUR, 0.005);
        conversion_fees.insert(Currency::GBP, 0.005);
        conversion_fees.insert(Currency::Bitcoin, 0.01);
        conversion_fees.insert(Currency::Ethereum, 0.008);
        conversion_fees.insert(Currency::ZephyrCoin, 0.0);

        Self {
            base_processing_fee: 0.005, // 0.5%
            conversion_fees,
            express_fee: 0.01, // 1%
            kyc_fee: 500, // $5 equivalent
        }
    }
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            batch_processing_enabled: true,
            batch_size: 100,
            processing_interval_minutes: 30,
            max_retries: 3,
            retry_delay_minutes: 60,
            auto_currency_conversion: true,
            fraud_detection_enabled: true,
        }
    }
}

impl PaymentProcessor {
    /// Create new payment processor
    pub fn new() -> Self {
        let mut processor = Self {
            payment_methods: HashMap::new(),
            exchange_rates: HashMap::new(),
            fee_structure: PaymentFeeStructure::default(),
            pending_payments: VecDeque::new(),
            payment_history: HashMap::new(),
            min_payout_thresholds: HashMap::new(),
            config: ProcessorConfig::default(),
        };

        processor.initialize_payment_methods();
        processor.initialize_exchange_rates();
        processor.initialize_payout_thresholds();

        processor
    }

    /// Initialize supported payment methods
    fn initialize_payment_methods(&mut self) {
        // Cryptocurrency methods
        self.payment_methods.insert(
            PaymentMethod::Cryptocurrency(CryptoNetwork::ZephyrCoin),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 1_000_000_000_000_000_000, // 1 ZEPH
                max_amount: 1_000_000 * 1_000_000_000_000_000_000, // 1M ZEPH
                processing_time_hours: 1,
                supported_currencies: vec![Currency::ZephyrCoin],
                geographic_restrictions: vec![],
                requires_kyc: false,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 0,
                    percentage_fee: 0.0,
                    network_fee: 100_000_000_000_000, // 0.0001 ZEPH
                    exchange_fee: 0.0,
                },
            },
        );

        self.payment_methods.insert(
            PaymentMethod::Cryptocurrency(CryptoNetwork::Ethereum),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 10_000_000_000_000_000, // 0.01 ETH
                max_amount: 1000 * 1_000_000_000_000_000_000, // 1000 ETH
                processing_time_hours: 1,
                supported_currencies: vec![Currency::Ethereum, Currency::USD],
                geographic_restrictions: vec![],
                requires_kyc: false,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 0,
                    percentage_fee: 0.003,
                    network_fee: 5_000_000_000_000_000, // ~$10 gas fee
                    exchange_fee: 0.005,
                },
            },
        );

        // Bank transfer methods
        self.payment_methods.insert(
            PaymentMethod::BankTransfer(BankTransferType::ACH),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 1000, // $10
                max_amount: 1_000_000_00, // $10,000
                processing_time_hours: 48,
                supported_currencies: vec![Currency::USD],
                geographic_restrictions: vec!["US".to_string()],
                requires_kyc: true,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 100, // $1
                    percentage_fee: 0.001,
                    network_fee: 0,
                    exchange_fee: 0.005,
                },
            },
        );

        self.payment_methods.insert(
            PaymentMethod::BankTransfer(BankTransferType::SEPA),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 1000, // €10
                max_amount: 1_000_000_00, // €10,000
                processing_time_hours: 24,
                supported_currencies: vec![Currency::EUR],
                geographic_restrictions: vec!["EU".to_string()],
                requires_kyc: true,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 50, // €0.50
                    percentage_fee: 0.001,
                    network_fee: 0,
                    exchange_fee: 0.005,
                },
            },
        );

        // Digital wallet methods
        self.payment_methods.insert(
            PaymentMethod::DigitalWallet(WalletProvider::PayPal),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 500, // $5
                max_amount: 500_000_00, // $5,000
                processing_time_hours: 2,
                supported_currencies: vec![Currency::USD, Currency::EUR, Currency::GBP],
                geographic_restrictions: vec![], // Global
                requires_kyc: true,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 30, // $0.30
                    percentage_fee: 0.029, // 2.9%
                    network_fee: 0,
                    exchange_fee: 0.035, // 3.5% for currency conversion
                },
            },
        );

        // Stablecoin methods
        self.payment_methods.insert(
            PaymentMethod::StableCoin(StableCoinType::USDC),
            PaymentMethodConfig {
                enabled: true,
                min_amount: 1_000_000, // 1 USDC
                max_amount: 100_000 * 1_000_000, // 100k USDC
                processing_time_hours: 1,
                supported_currencies: vec![Currency::USD],
                geographic_restrictions: vec![],
                requires_kyc: false,
                fee_structure: MethodFeeStructure {
                    fixed_fee: 0,
                    percentage_fee: 0.001,
                    network_fee: 2_000_000_000_000_000, // ~$2 gas fee
                    exchange_fee: 0.001,
                },
            },
        );
    }

    /// Initialize exchange rates (would fetch from APIs in production)
    fn initialize_exchange_rates(&mut self) {
        self.exchange_rates.insert(Currency::ZephyrCoin, 0.10); // $0.10 per ZEPH
        self.exchange_rates.insert(Currency::USD, 1.0);
        self.exchange_rates.insert(Currency::EUR, 0.85);
        self.exchange_rates.insert(Currency::GBP, 0.73);
        self.exchange_rates.insert(Currency::Bitcoin, 45000.0);
        self.exchange_rates.insert(Currency::Ethereum, 2500.0);
    }

    /// Initialize minimum payout thresholds
    fn initialize_payout_thresholds(&mut self) {
        self.min_payout_thresholds.insert(Currency::ZephyrCoin, 10 * 1_000_000_000_000_000_000); // 10 ZEPH
        self.min_payout_thresholds.insert(Currency::USD, 1000); // $10
        self.min_payout_thresholds.insert(Currency::EUR, 850); // €8.50
        self.min_payout_thresholds.insert(Currency::Bitcoin, 22222); // ~$10 worth
        self.min_payout_thresholds.insert(Currency::Ethereum, 400000); // ~$10 worth
    }

    /// Submit payment request
    pub fn submit_payment_request(&mut self, mut request: PaymentRequest) -> Result<String> {
        // Validate payment method
        let method_config = self.payment_methods.get(&request.payment_method)
            .ok_or_else(|| anyhow::anyhow!("Payment method not supported"))?;

        if !method_config.enabled {
            return Err(anyhow::anyhow!("Payment method temporarily disabled"));
        }

        // Convert amount to target currency
        let amount_in_currency = self.convert_tokens_to_currency(
            request.amount_tokens,
            &request.target_currency,
        )?;

        // Check minimum threshold
        if let Some(&min_threshold) = self.min_payout_thresholds.get(&request.target_currency) {
            if amount_in_currency < min_threshold {
                return Err(anyhow::anyhow!(
                    "Amount below minimum payout threshold: {} < {}",
                    amount_in_currency, min_threshold
                ));
            }
        }

        // Check method limits
        if amount_in_currency < method_config.min_amount || amount_in_currency > method_config.max_amount {
            return Err(anyhow::anyhow!(
                "Amount outside payment method limits: {} not in [{}, {}]",
                amount_in_currency, method_config.min_amount, method_config.max_amount
            ));
        }

        // Validate recipient info
        self.validate_recipient_info(&request.recipient_info, &request.payment_method)?;

        // Generate request ID
        request.request_id = format!("pay_{}_{}",
            chrono::Utc::now().timestamp(),
            &request.volunteer_id[..8]
        );

        self.pending_payments.push_back(request.clone());

        tracing::info!("Payment request submitted: {} for {} tokens",
            request.request_id, request.amount_tokens);

        Ok(request.request_id)
    }

    /// Convert tokens to target currency
    fn convert_tokens_to_currency(&self, tokens: u64, target_currency: &Currency) -> Result<u64> {
        if *target_currency == Currency::ZephyrCoin {
            return Ok(tokens);
        }

        let zeph_rate = self.exchange_rates.get(&Currency::ZephyrCoin)
            .ok_or_else(|| anyhow::anyhow!("ZephyrCoin exchange rate not available"))?;

        let target_rate = self.exchange_rates.get(target_currency)
            .ok_or_else(|| anyhow::anyhow!("Target currency exchange rate not available"))?;

        // Convert tokens to USD value, then to target currency
        let usd_value = (tokens as f64 / 1_000_000_000_000_000_000.0) * zeph_rate;
        let target_value = usd_value / target_rate;

        // Convert to smallest units (cents, wei, etc.)
        let target_amount = match target_currency {
            Currency::USD | Currency::EUR | Currency::GBP => (target_value * 100.0) as u64,
            Currency::Bitcoin => (target_value * 100_000_000.0) as u64, // Satoshis
            Currency::Ethereum => (target_value * 1_000_000_000_000_000_000.0) as u64, // Wei
            _ => (target_value * 1_000_000.0) as u64, // Default 6 decimals
        };

        Ok(target_amount)
    }

    /// Validate recipient information
    fn validate_recipient_info(&self, info: &RecipientInfo, method: &PaymentMethod) -> Result<()> {
        match method {
            PaymentMethod::Cryptocurrency(_) => {
                if info.wallet_address.is_none() {
                    return Err(anyhow::anyhow!("Wallet address required for crypto payments"));
                }
            },
            PaymentMethod::BankTransfer(_) => {
                if info.bank_account.is_none() {
                    return Err(anyhow::anyhow!("Bank account info required for bank transfers"));
                }
                if !info.kyc_verified {
                    return Err(anyhow::anyhow!("KYC verification required for bank transfers"));
                }
            },
            PaymentMethod::DigitalWallet(_) => {
                if info.digital_wallet.is_none() {
                    return Err(anyhow::anyhow!("Digital wallet info required"));
                }
            },
            PaymentMethod::StableCoin(_) => {
                if info.wallet_address.is_none() {
                    return Err(anyhow::anyhow!("Wallet address required for stablecoin payments"));
                }
            },
        }

        Ok(())
    }

    /// Process pending payments
    pub async fn process_pending_payments(&mut self) -> Result<Vec<PaymentRecord>> {
        let mut processed = Vec::new();
        let batch_size = if self.config.batch_processing_enabled {
            self.config.batch_size
        } else {
            1
        };

        for _ in 0..batch_size {
            if let Some(request) = self.pending_payments.pop_front() {
                // Check if scheduled for future
                if let Some(scheduled_time) = request.scheduled_for {
                    if Utc::now() < scheduled_time {
                        // Put back in queue
                        self.pending_payments.push_front(request);
                        break;
                    }
                }

                match self.process_single_payment(request).await {
                    Ok(record) => {
                        processed.push(record.clone());

                        // Add to history
                        self.payment_history
                            .entry(record.volunteer_id.clone())
                            .or_insert_with(Vec::new)
                            .push(record);
                    },
                    Err(e) => {
                        tracing::error!("Payment processing failed: {}", e);
                        // Could implement retry logic here
                    }
                }
            } else {
                break;
            }
        }

        Ok(processed)
    }

    /// Process single payment
    async fn process_single_payment(&self, request: PaymentRequest) -> Result<PaymentRecord> {
        let payment_id = format!("tx_{}_{}",
            chrono::Utc::now().timestamp_millis(),
            &request.volunteer_id[..6]
        );

        // Calculate fees
        let fees = self.calculate_payment_fees(&request)?;

        // Convert amount
        let amount_in_currency = self.convert_tokens_to_currency(
            request.amount_tokens,
            &request.target_currency,
        )?;

        let net_amount = amount_in_currency.saturating_sub(fees.total_fee);

        // Execute payment based on method
        let (status, confirmation_hash, error_message) = match request.payment_method {
            PaymentMethod::Cryptocurrency(_) => {
                self.execute_crypto_payment(&request, net_amount).await?
            },
            PaymentMethod::BankTransfer(_) => {
                self.execute_bank_transfer(&request, net_amount).await?
            },
            PaymentMethod::DigitalWallet(_) => {
                self.execute_wallet_payment(&request, net_amount).await?
            },
            PaymentMethod::StableCoin(_) => {
                self.execute_stablecoin_payment(&request, net_amount).await?
            },
        };

        let record = PaymentRecord {
            payment_id,
            request_id: request.request_id,
            volunteer_id: request.volunteer_id,
            amount_tokens: request.amount_tokens,
            amount_paid: net_amount,
            currency: request.target_currency,
            payment_method: request.payment_method,
            status,
            fees_paid: fees.total_fee,
            exchange_rate: self.get_exchange_rate(&Currency::ZephyrCoin, &request.target_currency)?,
            processed_at: Utc::now(),
            confirmation_hash,
            error_message,
        };

        tracing::info!("Payment processed: {} - {} {:?}",
            record.payment_id, record.amount_paid, record.currency);

        Ok(record)
    }

    /// Calculate payment fees
    fn calculate_payment_fees(&self, request: &PaymentRequest) -> Result<PaymentFees> {
        let method_config = self.payment_methods.get(&request.payment_method)
            .ok_or_else(|| anyhow::anyhow!("Payment method not found"))?;

        let amount_in_currency = self.convert_tokens_to_currency(
            request.amount_tokens,
            &request.target_currency,
        )?;

        let base_fee = (amount_in_currency as f64 * self.fee_structure.base_processing_fee) as u64;
        let method_percentage_fee = (amount_in_currency as f64 * method_config.fee_structure.percentage_fee) as u64;
        let method_fixed_fee = method_config.fee_structure.fixed_fee;
        let network_fee = method_config.fee_structure.network_fee;

        let exchange_fee = if request.target_currency != Currency::ZephyrCoin {
            let exchange_rate = method_config.fee_structure.exchange_fee;
            (amount_in_currency as f64 * exchange_rate) as u64
        } else {
            0
        };

        let express_fee = if matches!(request.priority, PaymentPriority::Express | PaymentPriority::Immediate) {
            (amount_in_currency as f64 * self.fee_structure.express_fee) as u64
        } else {
            0
        };

        let total_fee = base_fee + method_percentage_fee + method_fixed_fee + network_fee + exchange_fee + express_fee;

        Ok(PaymentFees {
            base_fee,
            method_percentage_fee,
            method_fixed_fee,
            network_fee,
            exchange_fee,
            express_fee,
            total_fee,
        })
    }

    /// Execute cryptocurrency payment
    async fn execute_crypto_payment(
        &self,
        request: &PaymentRequest,
        amount: u64,
    ) -> Result<(PaymentStatus, Option<String>, Option<String>)> {
        // Simulate crypto transaction
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let confirmation_hash = format!("0x{:x}", rand::random::<u64>());

        Ok((PaymentStatus::Completed, Some(confirmation_hash), None))
    }

    /// Execute bank transfer
    async fn execute_bank_transfer(
        &self,
        request: &PaymentRequest,
        amount: u64,
    ) -> Result<(PaymentStatus, Option<String>, Option<String>)> {
        // Simulate bank transfer processing
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let reference = format!("TXN{}", chrono::Utc::now().timestamp());

        Ok((PaymentStatus::Processing, Some(reference), None))
    }

    /// Execute digital wallet payment
    async fn execute_wallet_payment(
        &self,
        request: &PaymentRequest,
        amount: u64,
    ) -> Result<(PaymentStatus, Option<String>, Option<String>)> {
        // Simulate wallet payment
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let transaction_id = format!("WLT{}", chrono::Utc::now().timestamp());

        Ok((PaymentStatus::Completed, Some(transaction_id), None))
    }

    /// Execute stablecoin payment
    async fn execute_stablecoin_payment(
        &self,
        request: &PaymentRequest,
        amount: u64,
    ) -> Result<(PaymentStatus, Option<String>, Option<String>)> {
        // Simulate stablecoin transfer
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let tx_hash = format!("0x{:x}", rand::random::<u64>());

        Ok((PaymentStatus::Completed, Some(tx_hash), None))
    }

    /// Get exchange rate between currencies
    fn get_exchange_rate(&self, from: &Currency, to: &Currency) -> Result<f64> {
        if from == to {
            return Ok(1.0);
        }

        let from_rate = self.exchange_rates.get(from)
            .ok_or_else(|| anyhow::anyhow!("Exchange rate not found for {:?}", from))?;

        let to_rate = self.exchange_rates.get(to)
            .ok_or_else(|| anyhow::anyhow!("Exchange rate not found for {:?}", to))?;

        Ok(from_rate / to_rate)
    }

    /// Get payment history for volunteer
    pub fn get_payment_history(&self, volunteer_id: &str) -> Vec<&PaymentRecord> {
        self.payment_history.get(volunteer_id)
            .map(|records| records.iter().collect())
            .unwrap_or_default()
    }

    /// Update exchange rates
    pub fn update_exchange_rates(&mut self, rates: HashMap<Currency, f64>) {
        for (currency, rate) in rates {
            self.exchange_rates.insert(currency, rate);
        }
    }

    /// Get supported payment methods for region
    pub fn get_supported_methods(&self, country_code: &str) -> Vec<&PaymentMethod> {
        self.payment_methods.iter()
            .filter(|(_, config)| {
                config.enabled &&
                (config.geographic_restrictions.is_empty() ||
                 config.geographic_restrictions.contains(&country_code.to_string()))
            })
            .map(|(method, _)| method)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentFees {
    pub base_fee: u64,
    pub method_percentage_fee: u64,
    pub method_fixed_fee: u64,
    pub network_fee: u64,
    pub exchange_fee: u64,
    pub express_fee: u64,
    pub total_fee: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_processor_creation() {
        let processor = PaymentProcessor::new();
        assert!(!processor.payment_methods.is_empty());
        assert!(!processor.exchange_rates.is_empty());
    }

    #[tokio::test]
    async fn test_payment_submission() {
        let mut processor = PaymentProcessor::new();

        let request = PaymentRequest {
            request_id: String::new(),
            volunteer_id: "test_volunteer".to_string(),
            amount_tokens: 100 * 1_000_000_000_000_000_000, // 100 ZEPH
            target_currency: Currency::USD,
            payment_method: PaymentMethod::DigitalWallet(WalletProvider::PayPal),
            recipient_info: RecipientInfo {
                wallet_address: None,
                bank_account: None,
                digital_wallet: Some(DigitalWalletInfo {
                    provider: WalletProvider::PayPal,
                    wallet_id: "test@example.com".to_string(),
                    verified: true,
                }),
                kyc_verified: true,
                tax_info: None,
            },
            priority: PaymentPriority::Standard,
            created_at: Utc::now(),
            scheduled_for: None,
            metadata: HashMap::new(),
        };

        let request_id = processor.submit_payment_request(request).unwrap();
        assert!(!request_id.is_empty());
        assert_eq!(processor.pending_payments.len(), 1);
    }

    #[test]
    fn test_currency_conversion() {
        let processor = PaymentProcessor::new();
        let tokens = 100 * 1_000_000_000_000_000_000; // 100 ZEPH

        let usd_amount = processor.convert_tokens_to_currency(tokens, &Currency::USD).unwrap();
        assert_eq!(usd_amount, 1000); // $10.00 (100 ZEPH * $0.10 * 100 cents)

        let zeph_amount = processor.convert_tokens_to_currency(tokens, &Currency::ZephyrCoin).unwrap();
        assert_eq!(zeph_amount, tokens); // Same amount in ZEPH
    }
}
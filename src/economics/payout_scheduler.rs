//! Automated Payout Scheduling System
//!
//! Manages scheduled payouts for ZephyrFS volunteers with configurable intervals

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration, Weekday, TimeZone};
use tokio::time::{sleep, Duration as TokioDuration};

use super::payment_processor::{PaymentProcessor, PaymentRequest, Currency, PaymentMethod, RecipientInfo, PaymentPriority};
use super::earnings_calculator::EarningsCalculator;

/// Automated payout scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutScheduler {
    /// Volunteer payout preferences
    pub volunteer_preferences: HashMap<String, PayoutPreferences>,
    /// Scheduled payouts
    pub scheduled_payouts: BTreeMap<DateTime<Utc>, ScheduledPayout>,
    /// Payout policies and rules
    pub policies: PayoutPolicies,
    /// Accumulated earnings per volunteer
    pub accumulated_earnings: HashMap<String, AccumulatedEarnings>,
    /// Schedule configuration
    pub schedule_config: ScheduleConfig,
    /// Performance tracking
    pub payout_history: HashMap<String, Vec<PayoutEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutPreferences {
    pub volunteer_id: String,
    pub frequency: PayoutFrequency,
    pub preferred_currency: Currency,
    pub preferred_method: PaymentMethod,
    pub recipient_info: RecipientInfo,
    pub minimum_threshold: u64,
    pub auto_payout_enabled: bool,
    pub timezone: String,
    pub preferred_day: Option<Weekday>,
    pub preferred_hour: u8, // 0-23
    pub priority: PaymentPriority,
    pub split_payments: Option<SplitPaymentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayoutFrequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Manual, // Only manual payouts
    Threshold(u64), // Pay when threshold reached
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPaymentConfig {
    pub enabled: bool,
    pub primary_percentage: f64, // 0.0-1.0
    pub primary_method: PaymentMethod,
    pub secondary_method: PaymentMethod,
    pub secondary_currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPayout {
    pub payout_id: String,
    pub volunteer_id: String,
    pub amount_tokens: u64,
    pub target_currency: Currency,
    pub payment_method: PaymentMethod,
    pub recipient_info: RecipientInfo,
    pub scheduled_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub priority: PaymentPriority,
    pub recurring: bool,
    pub next_occurrence: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutPolicies {
    /// Minimum time between payouts (hours)
    pub min_payout_interval_hours: u32,
    /// Maximum accumulated earnings before forced payout
    pub max_accumulated_tokens: u64,
    /// Grace period for failed payments (hours)
    pub payment_retry_grace_hours: u32,
    /// Automatic threshold adjustment
    pub auto_adjust_thresholds: bool,
    /// Holiday/weekend handling
    pub holiday_handling: HolidayHandling,
    /// Risk management
    pub risk_controls: RiskControls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayHandling {
    pub skip_weekends: bool,
    pub skip_holidays: bool,
    pub advance_before_holiday: bool,
    pub supported_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskControls {
    pub max_daily_payout_per_volunteer: u64,
    pub max_total_daily_payouts: u64,
    pub suspicious_activity_threshold: f64,
    pub require_additional_verification: bool,
    pub fraud_detection_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccumulatedEarnings {
    pub volunteer_id: String,
    pub total_tokens: u64,
    pub last_payout: Option<DateTime<Utc>>,
    pub accumulation_start: DateTime<Utc>,
    pub daily_breakdown: HashMap<String, u64>, // date -> earnings
    pub bonus_tokens: u64,
    pub pending_taxes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub processing_interval_minutes: u32,
    pub lookahead_hours: u32,
    pub batch_processing: bool,
    pub max_concurrent_payouts: usize,
    pub retry_failed_payouts: bool,
    pub notification_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutEvent {
    pub event_id: String,
    pub payout_id: String,
    pub volunteer_id: String,
    pub event_type: PayoutEventType,
    pub amount: u64,
    pub currency: Currency,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayoutEventType {
    Scheduled,
    ThresholdReached,
    Manual,
    Emergency,
    Retry,
}

impl Default for PayoutPolicies {
    fn default() -> Self {
        Self {
            min_payout_interval_hours: 24,
            max_accumulated_tokens: 1000 * 1_000_000_000_000_000_000, // 1000 ZEPH
            payment_retry_grace_hours: 72,
            auto_adjust_thresholds: true,
            holiday_handling: HolidayHandling {
                skip_weekends: false,
                skip_holidays: true,
                advance_before_holiday: true,
                supported_regions: vec!["US".to_string(), "EU".to_string()],
            },
            risk_controls: RiskControls {
                max_daily_payout_per_volunteer: 10000 * 1_000_000_000_000_000_000, // 10k ZEPH
                max_total_daily_payouts: 100000 * 1_000_000_000_000_000_000, // 100k ZEPH
                suspicious_activity_threshold: 5.0, // 5x normal activity
                require_additional_verification: false,
                fraud_detection_enabled: true,
            },
        }
    }
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            processing_interval_minutes: 15,
            lookahead_hours: 24,
            batch_processing: true,
            max_concurrent_payouts: 10,
            retry_failed_payouts: true,
            notification_enabled: true,
        }
    }
}

impl PayoutScheduler {
    /// Create new payout scheduler
    pub fn new() -> Self {
        Self {
            volunteer_preferences: HashMap::new(),
            scheduled_payouts: BTreeMap::new(),
            policies: PayoutPolicies::default(),
            accumulated_earnings: HashMap::new(),
            schedule_config: ScheduleConfig::default(),
            payout_history: HashMap::new(),
        }
    }

    /// Set volunteer payout preferences
    pub fn set_volunteer_preferences(&mut self, preferences: PayoutPreferences) {
        let volunteer_id = preferences.volunteer_id.clone();
        self.volunteer_preferences.insert(volunteer_id.clone(), preferences);

        // Initialize accumulated earnings if needed
        if !self.accumulated_earnings.contains_key(&volunteer_id) {
            self.accumulated_earnings.insert(volunteer_id.clone(), AccumulatedEarnings {
                volunteer_id,
                total_tokens: 0,
                last_payout: None,
                accumulation_start: Utc::now(),
                daily_breakdown: HashMap::new(),
                bonus_tokens: 0,
                pending_taxes: 0,
            });
        }

        tracing::info!("Updated payout preferences for volunteer: {}", volunteer_id);
    }

    /// Add earnings to volunteer's accumulated total
    pub fn add_earnings(&mut self, volunteer_id: &str, tokens: u64, bonus_tokens: u64) -> Result<()> {
        let accumulated = self.accumulated_earnings.get_mut(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer not found in earnings tracker"))?;

        accumulated.total_tokens += tokens;
        accumulated.bonus_tokens += bonus_tokens;

        // Track daily breakdown
        let today = Utc::now().date_naive().to_string();
        *accumulated.daily_breakdown.entry(today).or_insert(0) += tokens;

        // Check if threshold payout should be triggered
        if let Some(preferences) = self.volunteer_preferences.get(volunteer_id) {
            if let PayoutFrequency::Threshold(threshold) = preferences.frequency {
                if accumulated.total_tokens >= threshold {
                    self.schedule_threshold_payout(volunteer_id)?;
                }
            }

            // Check maximum accumulation policy
            if accumulated.total_tokens >= self.policies.max_accumulated_tokens {
                self.schedule_emergency_payout(volunteer_id)?;
            }
        }

        tracing::debug!("Added {} tokens to {}, total: {}",
            tokens, volunteer_id, accumulated.total_tokens);

        Ok(())
    }

    /// Schedule threshold-based payout
    fn schedule_threshold_payout(&mut self, volunteer_id: &str) -> Result<()> {
        let preferences = self.volunteer_preferences.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer preferences not found"))?;

        let accumulated = self.accumulated_earnings.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Accumulated earnings not found"))?;

        let payout = ScheduledPayout {
            payout_id: format!("threshold_{}_{}", volunteer_id, Utc::now().timestamp()),
            volunteer_id: volunteer_id.to_string(),
            amount_tokens: accumulated.total_tokens,
            target_currency: preferences.preferred_currency.clone(),
            payment_method: preferences.preferred_method.clone(),
            recipient_info: preferences.recipient_info.clone(),
            scheduled_time: self.calculate_next_payout_time(preferences)?,
            created_at: Utc::now(),
            priority: preferences.priority.clone(),
            recurring: false,
            next_occurrence: None,
            metadata: HashMap::from([
                ("trigger".to_string(), "threshold".to_string()),
                ("threshold".to_string(), preferences.minimum_threshold.to_string()),
            ]),
        };

        self.scheduled_payouts.insert(payout.scheduled_time, payout);

        tracing::info!("Scheduled threshold payout for {}: {} tokens",
            volunteer_id, accumulated.total_tokens);

        Ok(())
    }

    /// Schedule emergency payout for max accumulation
    fn schedule_emergency_payout(&mut self, volunteer_id: &str) -> Result<()> {
        let preferences = self.volunteer_preferences.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer preferences not found"))?;

        let accumulated = self.accumulated_earnings.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Accumulated earnings not found"))?;

        let payout = ScheduledPayout {
            payout_id: format!("emergency_{}_{}", volunteer_id, Utc::now().timestamp()),
            volunteer_id: volunteer_id.to_string(),
            amount_tokens: accumulated.total_tokens,
            target_currency: preferences.preferred_currency.clone(),
            payment_method: preferences.preferred_method.clone(),
            recipient_info: preferences.recipient_info.clone(),
            scheduled_time: Utc::now() + Duration::hours(1), // Emergency: 1 hour delay
            created_at: Utc::now(),
            priority: PaymentPriority::Immediate,
            recurring: false,
            next_occurrence: None,
            metadata: HashMap::from([
                ("trigger".to_string(), "emergency".to_string()),
                ("reason".to_string(), "max_accumulation".to_string()),
            ]),
        };

        self.scheduled_payouts.insert(payout.scheduled_time, payout);

        tracing::warn!("Scheduled emergency payout for {}: {} tokens (max accumulation)",
            volunteer_id, accumulated.total_tokens);

        Ok(())
    }

    /// Calculate next payout time based on preferences
    fn calculate_next_payout_time(&self, preferences: &PayoutPreferences) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        let base_time = match preferences.frequency {
            PayoutFrequency::Daily => now + Duration::days(1),
            PayoutFrequency::Weekly => now + Duration::weeks(1),
            PayoutFrequency::BiWeekly => now + Duration::weeks(2),
            PayoutFrequency::Monthly => now + Duration::days(30),
            PayoutFrequency::Quarterly => now + Duration::days(90),
            PayoutFrequency::Threshold(_) => now + Duration::hours(1), // Immediate
            PayoutFrequency::Manual => return Err(anyhow::anyhow!("Manual payouts don't have scheduled times")),
        };

        // Adjust for preferred day and hour
        let mut adjusted_time = base_time;

        // Set preferred hour
        let target_hour = preferences.preferred_hour;
        adjusted_time = adjusted_time
            .with_hour(target_hour)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .ok_or_else(|| anyhow::anyhow!("Invalid time adjustment"))?;

        // Adjust for preferred day of week (for weekly/biweekly)
        if let Some(preferred_day) = preferences.preferred_day {
            if matches!(preferences.frequency, PayoutFrequency::Weekly | PayoutFrequency::BiWeekly) {
                let current_weekday = adjusted_time.weekday();
                let days_until_preferred = (preferred_day.number_from_monday() as i64
                    - current_weekday.number_from_monday() as i64 + 7) % 7;

                if days_until_preferred > 0 {
                    adjusted_time += Duration::days(days_until_preferred);
                }
            }
        }

        // Handle holidays and weekends
        adjusted_time = self.adjust_for_holidays(adjusted_time);

        Ok(adjusted_time)
    }

    /// Adjust payout time for holidays and weekends
    fn adjust_for_holidays(&self, mut payout_time: DateTime<Utc>) -> DateTime<Utc> {
        let holiday_config = &self.policies.holiday_handling;

        // Skip weekends if configured
        if holiday_config.skip_weekends {
            let weekday = payout_time.weekday();
            if weekday == Weekday::Sat {
                payout_time += Duration::days(2); // Move to Monday
            } else if weekday == Weekday::Sun {
                payout_time += Duration::days(1); // Move to Monday
            }
        }

        // Skip holidays (simplified - would use real holiday calendar in production)
        if holiday_config.skip_holidays {
            // Example: Skip December 25th
            if payout_time.month() == 12 && payout_time.day() == 25 {
                payout_time += Duration::days(1);
            }
        }

        payout_time
    }

    /// Generate recurring payouts for all volunteers
    pub fn generate_recurring_payouts(&mut self) -> Result<usize> {
        let mut created_count = 0;
        let now = Utc::now();
        let lookahead = now + Duration::hours(self.schedule_config.lookahead_hours as i64);

        for (volunteer_id, preferences) in &self.volunteer_preferences.clone() {
            if !preferences.auto_payout_enabled {
                continue;
            }

            let accumulated = self.accumulated_earnings.get(volunteer_id)
                .ok_or_else(|| anyhow::anyhow!("Accumulated earnings not found for {}", volunteer_id))?;

            // Check if volunteer has earnings to pay out
            if accumulated.total_tokens < preferences.minimum_threshold {
                continue;
            }

            // Check if last payout was recent enough
            if let Some(last_payout) = accumulated.last_payout {
                let hours_since_last = (now - last_payout).num_hours();
                if hours_since_last < self.policies.min_payout_interval_hours as i64 {
                    continue;
                }
            }

            // Calculate next payout time
            let next_payout_time = self.calculate_next_payout_time(preferences)?;

            // Only schedule if within lookahead window
            if next_payout_time <= lookahead {
                // Check if already scheduled
                let already_scheduled = self.scheduled_payouts.values()
                    .any(|payout| payout.volunteer_id == *volunteer_id && !payout.recurring);

                if !already_scheduled {
                    let payout = self.create_scheduled_payout(volunteer_id, preferences, accumulated)?;
                    self.scheduled_payouts.insert(payout.scheduled_time, payout);
                    created_count += 1;
                }
            }
        }

        if created_count > 0 {
            tracing::info!("Generated {} recurring payouts", created_count);
        }

        Ok(created_count)
    }

    /// Create scheduled payout from preferences and earnings
    fn create_scheduled_payout(
        &self,
        volunteer_id: &str,
        preferences: &PayoutPreferences,
        accumulated: &AccumulatedEarnings,
    ) -> Result<ScheduledPayout> {
        let payout_time = self.calculate_next_payout_time(preferences)?;

        // Handle split payments
        let (amount, method, currency) = if let Some(split_config) = &preferences.split_payments {
            if split_config.enabled {
                // For now, use primary payment - secondary would be handled separately
                let primary_amount = (accumulated.total_tokens as f64 * split_config.primary_percentage) as u64;
                (primary_amount, split_config.primary_method.clone(), preferences.preferred_currency.clone())
            } else {
                (accumulated.total_tokens, preferences.preferred_method.clone(), preferences.preferred_currency.clone())
            }
        } else {
            (accumulated.total_tokens, preferences.preferred_method.clone(), preferences.preferred_currency.clone())
        };

        Ok(ScheduledPayout {
            payout_id: format!("sched_{}_{}", volunteer_id, payout_time.timestamp()),
            volunteer_id: volunteer_id.to_string(),
            amount_tokens: amount,
            target_currency: currency,
            payment_method: method,
            recipient_info: preferences.recipient_info.clone(),
            scheduled_time: payout_time,
            created_at: Utc::now(),
            priority: preferences.priority.clone(),
            recurring: true,
            next_occurrence: Some(self.calculate_next_recurring_time(preferences, payout_time)?),
            metadata: HashMap::from([
                ("trigger".to_string(), "recurring".to_string()),
                ("frequency".to_string(), format!("{:?}", preferences.frequency)),
            ]),
        })
    }

    /// Calculate next occurrence for recurring payout
    fn calculate_next_recurring_time(
        &self,
        preferences: &PayoutPreferences,
        current_time: DateTime<Utc>,
    ) -> Result<DateTime<Utc>> {
        let next_time = match preferences.frequency {
            PayoutFrequency::Daily => current_time + Duration::days(1),
            PayoutFrequency::Weekly => current_time + Duration::weeks(1),
            PayoutFrequency::BiWeekly => current_time + Duration::weeks(2),
            PayoutFrequency::Monthly => current_time + Duration::days(30),
            PayoutFrequency::Quarterly => current_time + Duration::days(90),
            _ => return Err(anyhow::anyhow!("Frequency doesn't support recurring")),
        };

        Ok(self.adjust_for_holidays(next_time))
    }

    /// Process due payouts
    pub async fn process_due_payouts(
        &mut self,
        payment_processor: &mut PaymentProcessor,
    ) -> Result<Vec<PayoutEvent>> {
        let now = Utc::now();
        let mut events = Vec::new();

        // Collect due payouts
        let due_payouts: Vec<_> = self.scheduled_payouts
            .range(..=now)
            .map(|(_, payout)| payout.clone())
            .collect();

        for payout in due_payouts {
            // Remove from scheduled
            self.scheduled_payouts.remove(&payout.scheduled_time);

            // Process payout
            let event = self.process_single_payout(payout, payment_processor).await?;
            events.push(event.clone());

            // Record in history
            self.payout_history
                .entry(event.volunteer_id.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }

        Ok(events)
    }

    /// Process single payout
    async fn process_single_payout(
        &mut self,
        payout: ScheduledPayout,
        payment_processor: &mut PaymentProcessor,
    ) -> Result<PayoutEvent> {
        let event_id = format!("event_{}_{}", payout.volunteer_id, Utc::now().timestamp_millis());

        // Apply risk controls
        if let Err(risk_error) = self.check_risk_controls(&payout) {
            let event = PayoutEvent {
                event_id,
                payout_id: payout.payout_id,
                volunteer_id: payout.volunteer_id,
                event_type: PayoutEventType::Scheduled,
                amount: payout.amount_tokens,
                currency: payout.target_currency,
                timestamp: Utc::now(),
                success: false,
                error_message: Some(risk_error.to_string()),
                payment_reference: None,
            };

            tracing::warn!("Payout blocked by risk controls: {}", risk_error);
            return Ok(event);
        }

        // Create payment request
        let payment_request = PaymentRequest {
            request_id: payout.payout_id.clone(),
            volunteer_id: payout.volunteer_id.clone(),
            amount_tokens: payout.amount_tokens,
            target_currency: payout.target_currency.clone(),
            payment_method: payout.payment_method.clone(),
            recipient_info: payout.recipient_info.clone(),
            priority: payout.priority.clone(),
            created_at: Utc::now(),
            scheduled_for: None,
            metadata: payout.metadata.clone(),
        };

        // Submit to payment processor
        match payment_processor.submit_payment_request(payment_request) {
            Ok(payment_reference) => {
                // Update accumulated earnings
                if let Some(accumulated) = self.accumulated_earnings.get_mut(&payout.volunteer_id) {
                    accumulated.total_tokens = accumulated.total_tokens.saturating_sub(payout.amount_tokens);
                    accumulated.last_payout = Some(Utc::now());
                }

                // Schedule next occurrence if recurring
                if payout.recurring {
                    if let Some(next_time) = payout.next_occurrence {
                        let mut next_payout = payout.clone();
                        next_payout.payout_id = format!("sched_{}_{}", payout.volunteer_id, next_time.timestamp());
                        next_payout.scheduled_time = next_time;
                        next_payout.created_at = Utc::now();

                        // Calculate next occurrence after this one
                        if let Some(preferences) = self.volunteer_preferences.get(&payout.volunteer_id) {
                            next_payout.next_occurrence = self.calculate_next_recurring_time(preferences, next_time).ok();
                        }

                        self.scheduled_payouts.insert(next_time, next_payout);
                    }
                }

                let event = PayoutEvent {
                    event_id,
                    payout_id: payout.payout_id,
                    volunteer_id: payout.volunteer_id,
                    event_type: PayoutEventType::Scheduled,
                    amount: payout.amount_tokens,
                    currency: payout.target_currency,
                    timestamp: Utc::now(),
                    success: true,
                    error_message: None,
                    payment_reference: Some(payment_reference),
                };

                tracing::info!("Payout processed successfully: {} tokens to {}",
                    payout.amount_tokens, payout.volunteer_id);

                Ok(event)
            },
            Err(e) => {
                let event = PayoutEvent {
                    event_id,
                    payout_id: payout.payout_id,
                    volunteer_id: payout.volunteer_id,
                    event_type: PayoutEventType::Scheduled,
                    amount: payout.amount_tokens,
                    currency: payout.target_currency,
                    timestamp: Utc::now(),
                    success: false,
                    error_message: Some(e.to_string()),
                    payment_reference: None,
                };

                tracing::error!("Payout failed: {}", e);

                // Schedule retry if configured
                if self.schedule_config.retry_failed_payouts {
                    self.schedule_payout_retry(payout)?;
                }

                Ok(event)
            }
        }
    }

    /// Check risk controls for payout
    fn check_risk_controls(&self, payout: &ScheduledPayout) -> Result<()> {
        let risk_controls = &self.policies.risk_controls;

        // Check daily limit per volunteer
        if payout.amount_tokens > risk_controls.max_daily_payout_per_volunteer {
            return Err(anyhow::anyhow!("Exceeds daily payout limit per volunteer"));
        }

        // Check total daily payouts
        let today = Utc::now().date_naive();
        let today_payouts: u64 = self.payout_history
            .values()
            .flatten()
            .filter(|event| event.timestamp.date_naive() == today && event.success)
            .map(|event| event.amount)
            .sum();

        if today_payouts + payout.amount_tokens > risk_controls.max_total_daily_payouts {
            return Err(anyhow::anyhow!("Exceeds total daily payout limit"));
        }

        // Check for suspicious activity
        if risk_controls.fraud_detection_enabled {
            let volunteer_history = self.payout_history.get(&payout.volunteer_id);
            if let Some(history) = volunteer_history {
                if history.len() > 1 {
                    let recent_average = history.iter()
                        .rev()
                        .take(10)
                        .map(|e| e.amount)
                        .sum::<u64>() / 10.min(history.len()) as u64;

                    let activity_ratio = payout.amount_tokens as f64 / recent_average as f64;
                    if activity_ratio > risk_controls.suspicious_activity_threshold {
                        return Err(anyhow::anyhow!("Suspicious activity detected: {}x normal amount", activity_ratio));
                    }
                }
            }
        }

        Ok(())
    }

    /// Schedule retry for failed payout
    fn schedule_payout_retry(&mut self, mut payout: ScheduledPayout) -> Result<()> {
        let retry_time = Utc::now() + Duration::hours(self.policies.payment_retry_grace_hours as i64);

        payout.payout_id = format!("retry_{}_{}", payout.volunteer_id, retry_time.timestamp());
        payout.scheduled_time = retry_time;
        payout.priority = PaymentPriority::Express; // Higher priority for retries
        payout.metadata.insert("retry".to_string(), "true".to_string());

        self.scheduled_payouts.insert(retry_time, payout);

        Ok(())
    }

    /// Run automated payout processing loop
    pub async fn run_automated_processing(
        &mut self,
        mut payment_processor: PaymentProcessor,
        mut earnings_calculator: EarningsCalculator,
    ) -> Result<()> {
        let mut interval = tokio::time::interval(
            TokioDuration::from_secs(self.schedule_config.processing_interval_minutes as u64 * 60)
        );

        loop {
            interval.tick().await;

            // Generate recurring payouts
            if let Err(e) = self.generate_recurring_payouts() {
                tracing::error!("Failed to generate recurring payouts: {}", e);
            }

            // Process due payouts
            match self.process_due_payouts(&mut payment_processor).await {
                Ok(events) => {
                    if !events.is_empty() {
                        tracing::info!("Processed {} payouts", events.len());
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to process payouts: {}", e);
                }
            }

            // Update accumulated earnings from calculator
            // This would be integrated with the earnings calculator in a real implementation

            tracing::debug!("Payout processing cycle complete");
        }
    }

    /// Get payout history for volunteer
    pub fn get_payout_history(&self, volunteer_id: &str) -> Vec<&PayoutEvent> {
        self.payout_history.get(volunteer_id)
            .map(|events| events.iter().collect())
            .unwrap_or_default()
    }

    /// Get upcoming payouts for volunteer
    pub fn get_upcoming_payouts(&self, volunteer_id: &str) -> Vec<&ScheduledPayout> {
        self.scheduled_payouts.values()
            .filter(|payout| payout.volunteer_id == volunteer_id)
            .collect()
    }

    /// Manual payout trigger
    pub async fn trigger_manual_payout(
        &mut self,
        volunteer_id: &str,
        payment_processor: &mut PaymentProcessor,
    ) -> Result<PayoutEvent> {
        let preferences = self.volunteer_preferences.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Volunteer preferences not found"))?;

        let accumulated = self.accumulated_earnings.get(volunteer_id)
            .ok_or_else(|| anyhow::anyhow!("Accumulated earnings not found"))?;

        if accumulated.total_tokens < preferences.minimum_threshold {
            return Err(anyhow::anyhow!("Below minimum payout threshold"));
        }

        let payout = ScheduledPayout {
            payout_id: format!("manual_{}_{}", volunteer_id, Utc::now().timestamp()),
            volunteer_id: volunteer_id.to_string(),
            amount_tokens: accumulated.total_tokens,
            target_currency: preferences.preferred_currency.clone(),
            payment_method: preferences.preferred_method.clone(),
            recipient_info: preferences.recipient_info.clone(),
            scheduled_time: Utc::now(),
            created_at: Utc::now(),
            priority: PaymentPriority::Express,
            recurring: false,
            next_occurrence: None,
            metadata: HashMap::from([
                ("trigger".to_string(), "manual".to_string()),
            ]),
        };

        self.process_single_payout(payout, payment_processor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economics::payment_processor::{PaymentMethod, WalletProvider, RecipientInfo, DigitalWalletInfo};

    #[test]
    fn test_payout_scheduler_creation() {
        let scheduler = PayoutScheduler::new();
        assert!(scheduler.volunteer_preferences.is_empty());
        assert!(scheduler.scheduled_payouts.is_empty());
    }

    #[test]
    fn test_earnings_accumulation() {
        let mut scheduler = PayoutScheduler::new();

        // Set up volunteer preferences
        let preferences = PayoutPreferences {
            volunteer_id: "test_volunteer".to_string(),
            frequency: PayoutFrequency::Weekly,
            preferred_currency: Currency::USD,
            preferred_method: PaymentMethod::DigitalWallet(WalletProvider::PayPal),
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
            minimum_threshold: 10 * 1_000_000_000_000_000_000, // 10 ZEPH
            auto_payout_enabled: true,
            timezone: "UTC".to_string(),
            preferred_day: Some(Weekday::Fri),
            preferred_hour: 14,
            priority: PaymentPriority::Standard,
            split_payments: None,
        };

        scheduler.set_volunteer_preferences(preferences);

        // Add earnings
        scheduler.add_earnings("test_volunteer", 5 * 1_000_000_000_000_000_000, 0).unwrap();

        let accumulated = scheduler.accumulated_earnings.get("test_volunteer").unwrap();
        assert_eq!(accumulated.total_tokens, 5 * 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_threshold_payout_trigger() {
        let mut scheduler = PayoutScheduler::new();

        let preferences = PayoutPreferences {
            volunteer_id: "test_volunteer".to_string(),
            frequency: PayoutFrequency::Threshold(10 * 1_000_000_000_000_000_000), // 10 ZEPH threshold
            preferred_currency: Currency::ZephyrCoin,
            preferred_method: PaymentMethod::DigitalWallet(WalletProvider::PayPal),
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
            minimum_threshold: 10 * 1_000_000_000_000_000_000,
            auto_payout_enabled: true,
            timezone: "UTC".to_string(),
            preferred_day: None,
            preferred_hour: 12,
            priority: PaymentPriority::Standard,
            split_payments: None,
        };

        scheduler.set_volunteer_preferences(preferences);

        // Add earnings that exceed threshold
        scheduler.add_earnings("test_volunteer", 15 * 1_000_000_000_000_000_000, 0).unwrap();

        // Should have scheduled a payout
        assert!(!scheduler.scheduled_payouts.is_empty());
    }
}
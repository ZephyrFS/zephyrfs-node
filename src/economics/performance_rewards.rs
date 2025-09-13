//! Performance-Based Rewards System
//!
//! Advanced reward system that incentivizes excellence and network contribution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

use super::token_model::{RewardReason, NetworkHealthMetrics};
use super::earnings_calculator::{VolunteerMetrics, GeographicRegion, ConnectionQuality};

/// Performance-based rewards manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRewardsSystem {
    /// Performance scoring configuration
    pub scoring_config: ScoringConfiguration,
    /// Reward tier definitions
    pub reward_tiers: Vec<RewardTier>,
    /// Achievement system
    pub achievements: HashMap<String, Achievement>,
    /// Volunteer performance tracking
    pub volunteer_scores: HashMap<String, PerformanceScore>,
    /// Leaderboards and competitions
    pub leaderboards: HashMap<String, Leaderboard>,
    /// Special events and challenges
    pub active_challenges: HashMap<String, Challenge>,
    /// Reward multipliers and boosts
    pub active_multipliers: HashMap<String, RewardMultiplier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfiguration {
    /// Weight factors for different metrics
    pub uptime_weight: f64,
    pub speed_weight: f64,
    pub reliability_weight: f64,
    pub longevity_weight: f64,
    pub contribution_weight: f64,
    pub diversity_weight: f64,

    /// Performance windows
    pub daily_window_hours: u32,
    pub weekly_window_days: u32,
    pub monthly_window_days: u32,

    /// Scoring thresholds
    pub excellent_threshold: f64,
    pub good_threshold: f64,
    pub average_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTier {
    pub tier_name: String,
    pub min_score: f64,
    pub max_score: f64,
    pub multiplier: f64,
    pub badge_icon: String,
    pub color: String,
    pub benefits: Vec<TierBenefit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBenefit {
    pub benefit_type: BenefitType,
    pub value: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenefitType {
    EarningsMultiplier,
    PrioritySupport,
    EarlyAccess,
    ReducedFees,
    BonusPayouts,
    ExclusiveFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub rarity: AchievementRarity,
    pub requirements: AchievementRequirements,
    pub reward_tokens: u64,
    pub reward_multiplier: f64,
    pub one_time: bool,
    pub unlocked_by: Vec<String>, // volunteer IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementRequirements {
    pub min_uptime_percentage: Option<f64>,
    pub min_storage_gb: Option<u64>,
    pub min_speed_mbps: Option<f64>,
    pub min_reliability_score: Option<f64>,
    pub min_days_active: Option<u32>,
    pub geographic_requirements: Option<Vec<GeographicRegion>>,
    pub network_contribution: Option<f64>,
    pub custom_conditions: Vec<CustomCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCondition {
    pub condition_type: String,
    pub target_value: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    pub volunteer_id: String,
    pub overall_score: f64,
    pub component_scores: ComponentScores,
    pub tier: String,
    pub rank: u32,
    pub percentile: f64,
    pub trend: ScoreTrend,
    pub last_updated: DateTime<Utc>,
    pub achievements: Vec<String>,
    pub active_multipliers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub uptime_score: f64,
    pub speed_score: f64,
    pub reliability_score: f64,
    pub longevity_score: f64,
    pub contribution_score: f64,
    pub diversity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreTrend {
    Improving,
    Stable,
    Declining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub leaderboard_id: String,
    pub name: String,
    pub description: String,
    pub metric: LeaderboardMetric,
    pub timeframe: LeaderboardTimeframe,
    pub entries: Vec<LeaderboardEntry>,
    pub rewards: LeaderboardRewards,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaderboardMetric {
    OverallScore,
    Uptime,
    Speed,
    Reliability,
    StorageProvided,
    EarningsPerGB,
    NetworkContribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaderboardTimeframe {
    Daily,
    Weekly,
    Monthly,
    AllTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub volunteer_id: String,
    pub display_name: String,
    pub value: f64,
    pub tier: String,
    pub change_from_previous: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardRewards {
    pub top_1_reward: u64,
    pub top_5_reward: u64,
    pub top_10_reward: u64,
    pub top_50_reward: u64,
    pub participation_reward: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub challenge_id: String,
    pub name: String,
    pub description: String,
    pub challenge_type: ChallengeType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub requirements: ChallengeRequirements,
    pub rewards: ChallengeRewards,
    pub participants: HashMap<String, ChallengeProgress>,
    pub max_participants: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    UptimeChallenge,
    SpeedChallenge,
    StorageChallenge,
    CommunityChallenge,
    SpecialEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRequirements {
    pub target_metric: String,
    pub target_value: f64,
    pub duration_days: u32,
    pub min_participation_days: u32,
    pub geographic_restrictions: Option<Vec<GeographicRegion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRewards {
    pub completion_reward: u64,
    pub milestone_rewards: Vec<MilestoneReward>,
    pub leaderboard_rewards: Option<LeaderboardRewards>,
    pub exclusive_achievement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReward {
    pub milestone_percentage: f64,
    pub reward_tokens: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeProgress {
    pub volunteer_id: String,
    pub joined_at: DateTime<Utc>,
    pub current_progress: f64,
    pub milestones_achieved: Vec<u32>,
    pub daily_contributions: HashMap<String, f64>, // date -> contribution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMultiplier {
    pub multiplier_id: String,
    pub name: String,
    pub description: String,
    pub multiplier_value: f64,
    pub applicable_to: Vec<String>, // volunteer IDs or "all"
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub conditions: MultiplierConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplierConditions {
    pub min_tier: Option<String>,
    pub geographic_regions: Option<Vec<GeographicRegion>>,
    pub time_of_day: Option<(u8, u8)>, // (start_hour, end_hour)
    pub min_performance_score: Option<f64>,
    pub network_health_threshold: Option<f64>,
}

impl Default for ScoringConfiguration {
    fn default() -> Self {
        Self {
            uptime_weight: 0.25,
            speed_weight: 0.20,
            reliability_weight: 0.20,
            longevity_weight: 0.15,
            contribution_weight: 0.15,
            diversity_weight: 0.05,
            daily_window_hours: 24,
            weekly_window_days: 7,
            monthly_window_days: 30,
            excellent_threshold: 90.0,
            good_threshold: 75.0,
            average_threshold: 60.0,
        }
    }
}

impl PerformanceRewardsSystem {
    /// Create new performance rewards system
    pub fn new() -> Self {
        let mut system = Self {
            scoring_config: ScoringConfiguration::default(),
            reward_tiers: Vec::new(),
            achievements: HashMap::new(),
            volunteer_scores: HashMap::new(),
            leaderboards: HashMap::new(),
            active_challenges: HashMap::new(),
            active_multipliers: HashMap::new(),
        };

        system.initialize_reward_tiers();
        system.initialize_achievements();
        system.initialize_leaderboards();

        system
    }

    /// Initialize reward tiers
    fn initialize_reward_tiers(&mut self) {
        self.reward_tiers = vec![
            RewardTier {
                tier_name: "Diamond".to_string(),
                min_score: 95.0,
                max_score: 100.0,
                multiplier: 2.0,
                badge_icon: "💎".to_string(),
                color: "#B9F2FF".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 2.0,
                        description: "2x earnings multiplier".to_string(),
                    },
                    TierBenefit {
                        benefit_type: BenefitType::PrioritySupport,
                        value: 1.0,
                        description: "Priority customer support".to_string(),
                    },
                    TierBenefit {
                        benefit_type: BenefitType::ReducedFees,
                        value: 0.5,
                        description: "50% reduced fees".to_string(),
                    },
                ],
            },
            RewardTier {
                tier_name: "Platinum".to_string(),
                min_score: 85.0,
                max_score: 95.0,
                multiplier: 1.7,
                badge_icon: "🏆".to_string(),
                color: "#E5E4E2".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 1.7,
                        description: "1.7x earnings multiplier".to_string(),
                    },
                    TierBenefit {
                        benefit_type: BenefitType::EarlyAccess,
                        value: 1.0,
                        description: "Early access to new features".to_string(),
                    },
                ],
            },
            RewardTier {
                tier_name: "Gold".to_string(),
                min_score: 75.0,
                max_score: 85.0,
                multiplier: 1.4,
                badge_icon: "🥇".to_string(),
                color: "#FFD700".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 1.4,
                        description: "1.4x earnings multiplier".to_string(),
                    },
                    TierBenefit {
                        benefit_type: BenefitType::BonusPayouts,
                        value: 0.1,
                        description: "10% bonus payouts".to_string(),
                    },
                ],
            },
            RewardTier {
                tier_name: "Silver".to_string(),
                min_score: 60.0,
                max_score: 75.0,
                multiplier: 1.2,
                badge_icon: "🥈".to_string(),
                color: "#C0C0C0".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 1.2,
                        description: "1.2x earnings multiplier".to_string(),
                    },
                ],
            },
            RewardTier {
                tier_name: "Bronze".to_string(),
                min_score: 40.0,
                max_score: 60.0,
                multiplier: 1.0,
                badge_icon: "🥉".to_string(),
                color: "#CD7F32".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 1.0,
                        description: "Standard earnings".to_string(),
                    },
                ],
            },
            RewardTier {
                tier_name: "Newcomer".to_string(),
                min_score: 0.0,
                max_score: 40.0,
                multiplier: 0.8,
                badge_icon: "🌱".to_string(),
                color: "#90EE90".to_string(),
                benefits: vec![
                    TierBenefit {
                        benefit_type: BenefitType::EarningsMultiplier,
                        value: 0.8,
                        description: "Learning bonus - 80% earnings while improving".to_string(),
                    },
                ],
            },
        ];
    }

    /// Initialize achievements
    fn initialize_achievements(&mut self) {
        let achievements = vec![
            Achievement {
                achievement_id: "first_week".to_string(),
                name: "First Week Warrior".to_string(),
                description: "Complete your first week as a volunteer".to_string(),
                icon: "🎯".to_string(),
                rarity: AchievementRarity::Common,
                requirements: AchievementRequirements {
                    min_days_active: Some(7),
                    min_uptime_percentage: Some(80.0),
                    ..Default::default()
                },
                reward_tokens: 5 * 1_000_000_000_000_000_000, // 5 ZEPH
                reward_multiplier: 1.1,
                one_time: true,
                unlocked_by: Vec::new(),
            },
            Achievement {
                achievement_id: "speed_demon".to_string(),
                name: "Speed Demon".to_string(),
                description: "Achieve transfer speeds over 100 Mbps".to_string(),
                icon: "⚡".to_string(),
                rarity: AchievementRarity::Uncommon,
                requirements: AchievementRequirements {
                    min_speed_mbps: Some(100.0),
                    min_reliability_score: Some(95.0),
                    ..Default::default()
                },
                reward_tokens: 10 * 1_000_000_000_000_000_000, // 10 ZEPH
                reward_multiplier: 1.2,
                one_time: false,
                unlocked_by: Vec::new(),
            },
            Achievement {
                achievement_id: "reliability_champion".to_string(),
                name: "Reliability Champion".to_string(),
                description: "Maintain 99.9% uptime for 30 days".to_string(),
                icon: "🛡️".to_string(),
                rarity: AchievementRarity::Rare,
                requirements: AchievementRequirements {
                    min_uptime_percentage: Some(99.9),
                    min_days_active: Some(30),
                    min_reliability_score: Some(99.0),
                    ..Default::default()
                },
                reward_tokens: 25 * 1_000_000_000_000_000_000, // 25 ZEPH
                reward_multiplier: 1.5,
                one_time: false,
                unlocked_by: Vec::new(),
            },
            Achievement {
                achievement_id: "global_pioneer".to_string(),
                name: "Global Pioneer".to_string(),
                description: "First volunteer in an underrepresented region".to_string(),
                icon: "🌍".to_string(),
                rarity: AchievementRarity::Epic,
                requirements: AchievementRequirements {
                    geographic_requirements: Some(vec![GeographicRegion::Rare]),
                    min_days_active: Some(7),
                    ..Default::default()
                },
                reward_tokens: 50 * 1_000_000_000_000_000_000, // 50 ZEPH
                reward_multiplier: 2.0,
                one_time: true,
                unlocked_by: Vec::new(),
            },
            Achievement {
                achievement_id: "the_vault".to_string(),
                name: "The Vault".to_string(),
                description: "Provide 10TB of storage capacity".to_string(),
                icon: "🏛️".to_string(),
                rarity: AchievementRarity::Legendary,
                requirements: AchievementRequirements {
                    min_storage_gb: Some(10_000),
                    min_uptime_percentage: Some(95.0),
                    min_days_active: Some(90),
                    ..Default::default()
                },
                reward_tokens: 100 * 1_000_000_000_000_000_000, // 100 ZEPH
                reward_multiplier: 3.0,
                one_time: true,
                unlocked_by: Vec::new(),
            },
        ];

        for achievement in achievements {
            self.achievements.insert(achievement.achievement_id.clone(), achievement);
        }
    }

    /// Initialize leaderboards
    fn initialize_leaderboards(&mut self) {
        let leaderboards = vec![
            Leaderboard {
                leaderboard_id: "overall_weekly".to_string(),
                name: "Weekly Champions".to_string(),
                description: "Top performers this week".to_string(),
                metric: LeaderboardMetric::OverallScore,
                timeframe: LeaderboardTimeframe::Weekly,
                entries: Vec::new(),
                rewards: LeaderboardRewards {
                    top_1_reward: 50 * 1_000_000_000_000_000_000,
                    top_5_reward: 20 * 1_000_000_000_000_000_000,
                    top_10_reward: 10 * 1_000_000_000_000_000_000,
                    top_50_reward: 5 * 1_000_000_000_000_000_000,
                    participation_reward: 1 * 1_000_000_000_000_000_000,
                },
                last_updated: Utc::now(),
            },
            Leaderboard {
                leaderboard_id: "speed_monthly".to_string(),
                name: "Speed Masters".to_string(),
                description: "Fastest transfer speeds this month".to_string(),
                metric: LeaderboardMetric::Speed,
                timeframe: LeaderboardTimeframe::Monthly,
                entries: Vec::new(),
                rewards: LeaderboardRewards {
                    top_1_reward: 100 * 1_000_000_000_000_000_000,
                    top_5_reward: 40 * 1_000_000_000_000_000_000,
                    top_10_reward: 20 * 1_000_000_000_000_000_000,
                    top_50_reward: 10 * 1_000_000_000_000_000_000,
                    participation_reward: 2 * 1_000_000_000_000_000_000,
                },
                last_updated: Utc::now(),
            },
        ];

        for leaderboard in leaderboards {
            self.leaderboards.insert(leaderboard.leaderboard_id.clone(), leaderboard);
        }
    }

    /// Calculate comprehensive performance score
    pub fn calculate_performance_score(&self, metrics: &VolunteerMetrics, network_metrics: &NetworkHealthMetrics) -> Result<PerformanceScore> {
        let config = &self.scoring_config;

        // Calculate component scores
        let uptime_score = self.calculate_uptime_score(metrics)?;
        let speed_score = self.calculate_speed_score(metrics)?;
        let reliability_score = self.calculate_reliability_score(metrics)?;
        let longevity_score = self.calculate_longevity_score(metrics)?;
        let contribution_score = self.calculate_contribution_score(metrics, network_metrics)?;
        let diversity_score = self.calculate_diversity_score(metrics)?;

        let component_scores = ComponentScores {
            uptime_score,
            speed_score,
            reliability_score,
            longevity_score,
            contribution_score,
            diversity_score,
        };

        // Calculate weighted overall score
        let overall_score = uptime_score * config.uptime_weight
            + speed_score * config.speed_weight
            + reliability_score * config.reliability_weight
            + longevity_score * config.longevity_weight
            + contribution_score * config.contribution_weight
            + diversity_score * config.diversity_weight;

        // Determine tier
        let tier = self.determine_tier(overall_score);

        // Calculate trend (simplified - would use historical data)
        let trend = ScoreTrend::Stable;

        // Get achievements
        let achievements = self.check_achievements(metrics)?;

        // Get active multipliers
        let active_multipliers = self.get_active_multipliers(&metrics.volunteer_id);

        Ok(PerformanceScore {
            volunteer_id: metrics.volunteer_id.clone(),
            overall_score,
            component_scores,
            tier,
            rank: 0, // Would be calculated during ranking
            percentile: 0.0, // Would be calculated during ranking
            trend,
            last_updated: Utc::now(),
            achievements,
            active_multipliers,
        })
    }

    /// Calculate uptime score
    fn calculate_uptime_score(&self, metrics: &VolunteerMetrics) -> Result<f64> {
        // Perfect score at 99.9% uptime, linear scaling below
        let uptime = metrics.uptime_percentage;
        let score = if uptime >= 99.9 {
            100.0
        } else if uptime >= 95.0 {
            80.0 + (uptime - 95.0) * 4.0 // 80-100 range for 95-99.9%
        } else if uptime >= 80.0 {
            50.0 + (uptime - 80.0) * 2.0 // 50-80 range for 80-95%
        } else {
            uptime * 0.625 // 0-50 range for 0-80%
        };

        Ok(score.min(100.0))
    }

    /// Calculate speed score
    fn calculate_speed_score(&self, metrics: &VolunteerMetrics) -> Result<f64> {
        let speed = metrics.transfer_speed_mbps;
        let score = if speed >= 100.0 {
            100.0
        } else if speed >= 50.0 {
            75.0 + (speed - 50.0) * 0.5 // 75-100 for 50-100 Mbps
        } else if speed >= 10.0 {
            40.0 + (speed - 10.0) * 0.875 // 40-75 for 10-50 Mbps
        } else {
            speed * 4.0 // 0-40 for 0-10 Mbps
        };

        Ok(score.min(100.0))
    }

    /// Calculate reliability score
    fn calculate_reliability_score(&self, metrics: &VolunteerMetrics) -> Result<f64> {
        let total_transfers = metrics.successful_transfers + metrics.failed_transfers;
        if total_transfers == 0 {
            return Ok(100.0); // New volunteers get perfect score
        }

        let success_rate = metrics.successful_transfers as f64 / total_transfers as f64;
        let response_time_factor = if metrics.response_time_ms <= 100 {
            1.0
        } else if metrics.response_time_ms <= 500 {
            0.9
        } else {
            0.8
        };

        let score = success_rate * 100.0 * response_time_factor;
        Ok(score.min(100.0))
    }

    /// Calculate longevity score
    fn calculate_longevity_score(&self, metrics: &VolunteerMetrics) -> Result<f64> {
        let days_active = (Utc::now() - metrics.joined_at).num_days();
        let score = if days_active >= 365 {
            100.0
        } else if days_active >= 90 {
            70.0 + (days_active - 90) as f64 * 30.0 / 275.0
        } else if days_active >= 30 {
            40.0 + (days_active - 30) as f64 * 30.0 / 60.0
        } else if days_active >= 7 {
            20.0 + (days_active - 7) as f64 * 20.0 / 23.0
        } else {
            days_active as f64 * 20.0 / 7.0
        };

        Ok(score.min(100.0))
    }

    /// Calculate contribution score
    fn calculate_contribution_score(&self, metrics: &VolunteerMetrics, network_metrics: &NetworkHealthMetrics) -> Result<f64> {
        if network_metrics.total_capacity_gb == 0 {
            return Ok(0.0);
        }

        let contribution_percentage = metrics.total_storage_gb as f64 / network_metrics.total_capacity_gb as f64 * 100.0;
        let utilization_factor = if metrics.total_storage_gb > 0 {
            metrics.used_storage_gb as f64 / metrics.total_storage_gb as f64
        } else {
            0.0
        };

        // Score based on both absolute contribution and utilization
        let base_score = contribution_percentage * 1000.0; // Scale up
        let utilization_bonus = utilization_factor * 20.0; // Up to 20 point bonus

        let score = (base_score + utilization_bonus).min(100.0);
        Ok(score)
    }

    /// Calculate diversity score
    fn calculate_diversity_score(&self, metrics: &VolunteerMetrics) -> Result<f64> {
        let region_bonus = match metrics.geographic_region {
            GeographicRegion::Rare => 100.0,
            GeographicRegion::Africa => 80.0,
            GeographicRegion::SouthAmerica => 70.0,
            GeographicRegion::Oceania => 70.0,
            GeographicRegion::Asia => 60.0,
            GeographicRegion::Europe => 40.0,
            GeographicRegion::NorthAmerica => 30.0,
        };

        let connection_bonus = match metrics.connection_quality {
            ConnectionQuality::Excellent => 20.0,
            ConnectionQuality::Good => 15.0,
            ConnectionQuality::Fair => 10.0,
            ConnectionQuality::Poor => 5.0,
        };

        Ok((region_bonus + connection_bonus).min(100.0))
    }

    /// Determine performance tier
    fn determine_tier(&self, score: f64) -> String {
        for tier in &self.reward_tiers {
            if score >= tier.min_score && score <= tier.max_score {
                return tier.tier_name.clone();
            }
        }
        "Unranked".to_string()
    }

    /// Check for new achievements
    fn check_achievements(&self, metrics: &VolunteerMetrics) -> Result<Vec<String>> {
        let mut unlocked_achievements = Vec::new();

        for (achievement_id, achievement) in &self.achievements {
            if achievement.unlocked_by.contains(&metrics.volunteer_id) && achievement.one_time {
                continue; // Already unlocked
            }

            let mut meets_requirements = true;
            let reqs = &achievement.requirements;

            // Check uptime requirement
            if let Some(min_uptime) = reqs.min_uptime_percentage {
                if metrics.uptime_percentage < min_uptime {
                    meets_requirements = false;
                }
            }

            // Check storage requirement
            if let Some(min_storage) = reqs.min_storage_gb {
                if metrics.total_storage_gb < min_storage {
                    meets_requirements = false;
                }
            }

            // Check speed requirement
            if let Some(min_speed) = reqs.min_speed_mbps {
                if metrics.transfer_speed_mbps < min_speed {
                    meets_requirements = false;
                }
            }

            // Check reliability requirement
            if let Some(min_reliability) = reqs.min_reliability_score {
                if metrics.reliability_score < min_reliability {
                    meets_requirements = false;
                }
            }

            // Check days active requirement
            if let Some(min_days) = reqs.min_days_active {
                let days_active = (Utc::now() - metrics.joined_at).num_days();
                if days_active < min_days as i64 {
                    meets_requirements = false;
                }
            }

            // Check geographic requirements
            if let Some(ref required_regions) = reqs.geographic_requirements {
                if !required_regions.contains(&metrics.geographic_region) {
                    meets_requirements = false;
                }
            }

            if meets_requirements {
                unlocked_achievements.push(achievement_id.clone());
            }
        }

        Ok(unlocked_achievements)
    }

    /// Get active multipliers for volunteer
    fn get_active_multipliers(&self, volunteer_id: &str) -> Vec<String> {
        let now = Utc::now();

        self.active_multipliers
            .values()
            .filter(|multiplier| {
                // Check if still active
                if now < multiplier.start_time || now > multiplier.end_time {
                    return false;
                }

                // Check if applies to this volunteer
                multiplier.applicable_to.contains(&"all".to_string()) ||
                multiplier.applicable_to.contains(volunteer_id)
            })
            .map(|multiplier| multiplier.multiplier_id.clone())
            .collect()
    }

    /// Update volunteer score
    pub fn update_volunteer_score(&mut self, volunteer_id: String, score: PerformanceScore) {
        self.volunteer_scores.insert(volunteer_id, score);
    }

    /// Calculate reward multiplier for volunteer
    pub fn calculate_reward_multiplier(&self, volunteer_id: &str) -> f64 {
        let base_multiplier = if let Some(score) = self.volunteer_scores.get(volunteer_id) {
            // Get tier multiplier
            let tier_multiplier = self.reward_tiers
                .iter()
                .find(|tier| tier.tier_name == score.tier)
                .map(|tier| tier.multiplier)
                .unwrap_or(1.0);

            // Add achievement multipliers
            let achievement_multiplier: f64 = score.achievements
                .iter()
                .filter_map(|achievement_id| self.achievements.get(achievement_id))
                .map(|achievement| achievement.reward_multiplier - 1.0)
                .sum::<f64>() + 1.0;

            tier_multiplier * achievement_multiplier
        } else {
            1.0
        };

        // Apply active multipliers
        let active_multiplier: f64 = self.active_multipliers
            .values()
            .filter(|multiplier| {
                let now = Utc::now();
                now >= multiplier.start_time && now <= multiplier.end_time &&
                (multiplier.applicable_to.contains(&"all".to_string()) ||
                 multiplier.applicable_to.contains(volunteer_id))
            })
            .map(|multiplier| multiplier.multiplier_value)
            .product();

        base_multiplier * active_multiplier
    }

    /// Update leaderboards
    pub fn update_leaderboards(&mut self) -> Result<()> {
        for leaderboard in self.leaderboards.values_mut() {
            let mut entries: Vec<LeaderboardEntry> = self.volunteer_scores
                .values()
                .map(|score| {
                    let value = match leaderboard.metric {
                        LeaderboardMetric::OverallScore => score.overall_score,
                        LeaderboardMetric::Uptime => score.component_scores.uptime_score,
                        LeaderboardMetric::Speed => score.component_scores.speed_score,
                        LeaderboardMetric::Reliability => score.component_scores.reliability_score,
                        LeaderboardMetric::StorageProvided => score.component_scores.contribution_score,
                        LeaderboardMetric::EarningsPerGB => score.overall_score, // Simplified
                        LeaderboardMetric::NetworkContribution => score.component_scores.contribution_score,
                    };

                    LeaderboardEntry {
                        rank: 0, // Will be set after sorting
                        volunteer_id: score.volunteer_id.clone(),
                        display_name: format!("Volunteer_{}", &score.volunteer_id[..8]),
                        value,
                        tier: score.tier.clone(),
                        change_from_previous: 0, // Would track changes
                    }
                })
                .collect();

            // Sort by value (descending)
            entries.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));

            // Assign ranks
            for (index, entry) in entries.iter_mut().enumerate() {
                entry.rank = (index + 1) as u32;
            }

            leaderboard.entries = entries;
            leaderboard.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Create new challenge
    pub fn create_challenge(&mut self, challenge: Challenge) -> Result<()> {
        let challenge_id = challenge.challenge_id.clone();
        self.active_challenges.insert(challenge_id, challenge);
        Ok(())
    }

    /// Add reward multiplier
    pub fn add_reward_multiplier(&mut self, multiplier: RewardMultiplier) -> Result<()> {
        let multiplier_id = multiplier.multiplier_id.clone();
        self.active_multipliers.insert(multiplier_id, multiplier);
        Ok(())
    }

    /// Get volunteer's current tier and benefits
    pub fn get_volunteer_tier_info(&self, volunteer_id: &str) -> Option<(String, Vec<TierBenefit>)> {
        let score = self.volunteer_scores.get(volunteer_id)?;
        let tier = self.reward_tiers
            .iter()
            .find(|t| t.tier_name == score.tier)?;

        Some((tier.tier_name.clone(), tier.benefits.clone()))
    }

    /// Get leaderboard for display
    pub fn get_leaderboard(&self, leaderboard_id: &str) -> Option<&Leaderboard> {
        self.leaderboards.get(leaderboard_id)
    }
}

impl Default for AchievementRequirements {
    fn default() -> Self {
        Self {
            min_uptime_percentage: None,
            min_storage_gb: None,
            min_speed_mbps: None,
            min_reliability_score: None,
            min_days_active: None,
            geographic_requirements: None,
            network_contribution: None,
            custom_conditions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_rewards_system() {
        let system = PerformanceRewardsSystem::new();
        assert!(!system.reward_tiers.is_empty());
        assert!(!system.achievements.is_empty());
        assert!(!system.leaderboards.is_empty());
    }

    #[test]
    fn test_score_calculation() {
        let system = PerformanceRewardsSystem::new();

        let metrics = VolunteerMetrics {
            volunteer_id: "test_volunteer".to_string(),
            total_storage_gb: 100,
            available_storage_gb: 50,
            used_storage_gb: 50,
            uptime_hours_24h: 24.0,
            uptime_percentage: 99.5,
            response_time_ms: 50,
            transfer_speed_mbps: 100.0,
            successful_transfers: 1000,
            failed_transfers: 5,
            geographic_region: GeographicRegion::Europe,
            connection_quality: ConnectionQuality::Excellent,
            reliability_score: 99.5,
            last_seen: Utc::now(),
            joined_at: Utc::now() - Duration::days(365),
        };

        let network_metrics = NetworkHealthMetrics {
            total_capacity_gb: 10000,
            active_volunteers: 100,
            utilization_rate: 75.0,
            average_uptime: 95.0,
            geographic_diversity: 70.0,
            data_durability: 99.9,
        };

        let score = system.calculate_performance_score(&metrics, &network_metrics).unwrap();
        assert!(score.overall_score > 80.0); // Should be high-performing
        assert_eq!(score.tier, "Platinum"); // Should be in a high tier
    }

    #[test]
    fn test_achievement_checking() {
        let system = PerformanceRewardsSystem::new();

        let high_speed_metrics = VolunteerMetrics {
            volunteer_id: "speed_test".to_string(),
            total_storage_gb: 100,
            available_storage_gb: 0,
            used_storage_gb: 100,
            uptime_hours_24h: 24.0,
            uptime_percentage: 99.0,
            response_time_ms: 25,
            transfer_speed_mbps: 150.0,
            successful_transfers: 1000,
            failed_transfers: 1,
            geographic_region: GeographicRegion::NorthAmerica,
            connection_quality: ConnectionQuality::Excellent,
            reliability_score: 99.9,
            last_seen: Utc::now(),
            joined_at: Utc::now() - Duration::days(30),
        };

        let achievements = system.check_achievements(&high_speed_metrics).unwrap();
        assert!(achievements.contains(&"speed_demon".to_string()));
    }
}
//! Resource Scheduler
//!
//! Schedules resource allocation based on contribution priorities and system capacity

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Resource scheduler for contribution-based allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceScheduler {
    pub scheduled_allocations: Vec<ScheduledAllocation>,
    pub allocation_schedule: AllocationSchedule,
    pub scheduling_policies: Vec<SchedulingPolicy>,
    pub resource_reservations: HashMap<String, ResourceReservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAllocation {
    pub allocation_id: String,
    pub user_id: String,
    pub scheduled_for: DateTime<Utc>,
    pub duration: Option<Duration>,
    pub priority: u32,
    pub resource_requirements: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSchedule {
    pub schedule_id: String,
    pub time_slots: Vec<TimeSlot>,
    pub capacity_limits: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub available_capacity: HashMap<String, u64>,
    pub scheduled_allocations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingPolicy {
    pub policy_name: String,
    pub description: String,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservation {
    pub reservation_id: String,
    pub user_id: String,
    pub resource_type: String,
    pub amount: u64,
    pub reserved_until: DateTime<Utc>,
    pub status: ReservationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReservationStatus {
    Active,
    Pending,
    Expired,
    Cancelled,
}

impl ResourceScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_allocations: Vec::new(),
            allocation_schedule: AllocationSchedule {
                schedule_id: "default".to_string(),
                time_slots: Vec::new(),
                capacity_limits: HashMap::new(),
                last_updated: Utc::now(),
            },
            scheduling_policies: Vec::new(),
            resource_reservations: HashMap::new(),
        }
    }
}

impl Default for ResourceScheduler {
    fn default() -> Self {
        Self::new()
    }
}
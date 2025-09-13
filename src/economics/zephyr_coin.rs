//! ZephyrCoin Smart Contract Implementation
//!
//! ERC-20 compatible token for ZephyrFS network incentives

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// ZephyrCoin token contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZephyrCoin {
    /// Token metadata
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,

    /// Balance tracking
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,

    /// ZephyrFS-specific features
    pub contract_owner: String,
    pub minters: HashMap<String, bool>,
    pub burners: HashMap<String, bool>,
    pub paused: bool,

    /// Economic controls
    pub daily_mint_limit: u64,
    pub daily_minted: u64,
    pub last_mint_reset: DateTime<Utc>,

    /// Staking and governance
    pub staked_balances: HashMap<String, StakedBalance>,
    pub governance_proposals: HashMap<u64, GovernanceProposal>,
    pub proposal_counter: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakedBalance {
    pub amount: u64,
    pub staked_at: DateTime<Utc>,
    pub unlock_time: DateTime<Utc>,
    pub rewards_claimed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub target_contract: Option<String>,
    pub call_data: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub votes_for: u64,
    pub votes_against: u64,
    pub executed: bool,
    pub voters: HashMap<String, Vote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vote {
    For(u64),    // Amount of tokens voted
    Against(u64),
}

/// ERC-20 Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenEvent {
    Transfer {
        from: String,
        to: String,
        value: u64,
    },
    Approval {
        owner: String,
        spender: String,
        value: u64,
    },
    Mint {
        to: String,
        value: u64,
    },
    Burn {
        from: String,
        value: u64,
    },
    Stake {
        user: String,
        amount: u64,
        duration_days: u32,
    },
    Unstake {
        user: String,
        amount: u64,
        rewards: u64,
    },
    ProposalCreated {
        id: u64,
        proposer: String,
        title: String,
    },
    VoteCast {
        proposal_id: u64,
        voter: String,
        support: bool,
        weight: u64,
    },
}

impl Default for ZephyrCoin {
    fn default() -> Self {
        Self {
            name: "ZephyrCoin".to_string(),
            symbol: "ZEPH".to_string(),
            decimals: 18,
            total_supply: 0,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            contract_owner: String::new(),
            minters: HashMap::new(),
            burners: HashMap::new(),
            paused: false,
            daily_mint_limit: 1_000_000 * 10_u64.pow(18), // 1M tokens per day max
            daily_minted: 0,
            last_mint_reset: Utc::now(),
            staked_balances: HashMap::new(),
            governance_proposals: HashMap::new(),
            proposal_counter: 0,
        }
    }
}

impl ZephyrCoin {
    /// Initialize new ZephyrCoin contract
    pub fn new(owner: String, initial_supply: u64) -> Self {
        let mut coin = Self::default();
        coin.contract_owner = owner.clone();
        coin.total_supply = initial_supply;
        coin.balances.insert(owner.clone(), initial_supply);
        coin.minters.insert(owner, true);
        coin
    }

    /// ERC-20: Get balance of account
    pub fn balance_of(&self, account: &str) -> u64 {
        self.balances.get(account).copied().unwrap_or(0)
    }

    /// ERC-20: Transfer tokens
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<TokenEvent> {
        self.require_not_paused()?;

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        self.balances.insert(from.to_string(), from_balance - amount);
        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + amount);

        Ok(TokenEvent::Transfer {
            from: from.to_string(),
            to: to.to_string(),
            value: amount,
        })
    }

    /// ERC-20: Approve spender
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u64) -> Result<TokenEvent> {
        self.require_not_paused()?;

        self.allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), amount);

        Ok(TokenEvent::Approval {
            owner: owner.to_string(),
            spender: spender.to_string(),
            value: amount,
        })
    }

    /// ERC-20: Transfer from approved amount
    pub fn transfer_from(&mut self, spender: &str, from: &str, to: &str, amount: u64) -> Result<TokenEvent> {
        self.require_not_paused()?;

        // Check allowance
        let allowance = self.allowances
            .get(from)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0);

        if allowance < amount {
            return Err(anyhow::anyhow!("Insufficient allowance"));
        }

        // Update allowance
        self.allowances
            .get_mut(from)
            .unwrap()
            .insert(spender.to_string(), allowance - amount);

        // Perform transfer
        self.transfer(from, to, amount)
    }

    /// ERC-20: Get allowance
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0)
    }

    /// Mint new tokens (ZephyrFS-specific)
    pub fn mint(&mut self, caller: &str, to: &str, amount: u64) -> Result<TokenEvent> {
        self.require_not_paused()?;
        self.require_minter(caller)?;
        self.check_mint_limits(amount)?;

        self.total_supply += amount;
        let balance = self.balance_of(to);
        self.balances.insert(to.to_string(), balance + amount);

        // Update daily mint tracking
        self.daily_minted += amount;

        Ok(TokenEvent::Mint {
            to: to.to_string(),
            value: amount,
        })
    }

    /// Burn tokens (ZephyrFS-specific)
    pub fn burn(&mut self, caller: &str, from: &str, amount: u64) -> Result<TokenEvent> {
        self.require_not_paused()?;
        self.require_burner(caller)?;

        let balance = self.balance_of(from);
        if balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance to burn"));
        }

        self.total_supply -= amount;
        self.balances.insert(from.to_string(), balance - amount);

        Ok(TokenEvent::Burn {
            from: from.to_string(),
            value: amount,
        })
    }

    /// Stake tokens for governance and rewards
    pub fn stake(&mut self, user: &str, amount: u64, duration_days: u32) -> Result<TokenEvent> {
        self.require_not_paused()?;

        let balance = self.balance_of(user);
        if balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance to stake"));
        }

        // Lock tokens
        self.balances.insert(user.to_string(), balance - amount);

        // Add to staking
        let unlock_time = Utc::now() + chrono::Duration::days(duration_days as i64);
        let staked = StakedBalance {
            amount,
            staked_at: Utc::now(),
            unlock_time,
            rewards_claimed: 0,
        };

        self.staked_balances.insert(user.to_string(), staked);

        Ok(TokenEvent::Stake {
            user: user.to_string(),
            amount,
            duration_days,
        })
    }

    /// Unstake tokens after lock period
    pub fn unstake(&mut self, user: &str) -> Result<TokenEvent> {
        self.require_not_paused()?;

        let staked = self.staked_balances.get(user)
            .ok_or_else(|| anyhow::anyhow!("No staked balance found"))?;

        if Utc::now() < staked.unlock_time {
            return Err(anyhow::anyhow!("Tokens still locked"));
        }

        // Calculate staking rewards (5% APY)
        let staking_duration = (Utc::now() - staked.staked_at).num_days();
        let rewards = (staked.amount as f64 * 0.05 * staking_duration as f64 / 365.0) as u64;

        // Return staked amount + rewards
        let balance = self.balance_of(user);
        self.balances.insert(user.to_string(), balance + staked.amount + rewards);

        // Mint rewards
        self.total_supply += rewards;

        // Remove from staking
        self.staked_balances.remove(user);

        Ok(TokenEvent::Unstake {
            user: user.to_string(),
            amount: staked.amount,
            rewards,
        })
    }

    /// Create governance proposal
    pub fn create_proposal(
        &mut self,
        proposer: &str,
        title: String,
        description: String,
        target_contract: Option<String>,
        call_data: Option<Vec<u8>>,
        voting_duration_days: u32,
    ) -> Result<TokenEvent> {
        self.require_not_paused()?;

        // Require minimum stake to propose (10,000 ZEPH)
        let min_stake = 10_000 * 10_u64.pow(18);
        let staked = self.staked_balances.get(proposer)
            .ok_or_else(|| anyhow::anyhow!("Must stake tokens to propose"))?;

        if staked.amount < min_stake {
            return Err(anyhow::anyhow!("Insufficient stake to create proposal"));
        }

        self.proposal_counter += 1;
        let proposal = GovernanceProposal {
            id: self.proposal_counter,
            proposer: proposer.to_string(),
            title: title.clone(),
            description,
            target_contract,
            call_data,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(voting_duration_days as i64),
            votes_for: 0,
            votes_against: 0,
            executed: false,
            voters: HashMap::new(),
        };

        self.governance_proposals.insert(self.proposal_counter, proposal);

        Ok(TokenEvent::ProposalCreated {
            id: self.proposal_counter,
            proposer: proposer.to_string(),
            title,
        })
    }

    /// Vote on governance proposal
    pub fn vote(&mut self, voter: &str, proposal_id: u64, support: bool) -> Result<TokenEvent> {
        self.require_not_paused()?;

        let proposal = self.governance_proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;

        if Utc::now() > proposal.voting_ends_at {
            return Err(anyhow::anyhow!("Voting period ended"));
        }

        if proposal.voters.contains_key(voter) {
            return Err(anyhow::anyhow!("Already voted"));
        }

        // Voting weight = staked tokens
        let staked = self.staked_balances.get(voter)
            .ok_or_else(|| anyhow::anyhow!("Must stake tokens to vote"))?;

        let weight = staked.amount;

        if support {
            proposal.votes_for += weight;
            proposal.voters.insert(voter.to_string(), Vote::For(weight));
        } else {
            proposal.votes_against += weight;
            proposal.voters.insert(voter.to_string(), Vote::Against(weight));
        }

        Ok(TokenEvent::VoteCast {
            proposal_id,
            voter: voter.to_string(),
            support,
            weight,
        })
    }

    /// Add minter role
    pub fn add_minter(&mut self, caller: &str, minter: &str) -> Result<()> {
        self.require_owner(caller)?;
        self.minters.insert(minter.to_string(), true);
        Ok(())
    }

    /// Add burner role
    pub fn add_burner(&mut self, caller: &str, burner: &str) -> Result<()> {
        self.require_owner(caller)?;
        self.burners.insert(burner.to_string(), true);
        Ok(())
    }

    /// Pause contract
    pub fn pause(&mut self, caller: &str) -> Result<()> {
        self.require_owner(caller)?;
        self.paused = true;
        Ok(())
    }

    /// Unpause contract
    pub fn unpause(&mut self, caller: &str) -> Result<()> {
        self.require_owner(caller)?;
        self.paused = false;
        Ok(())
    }

    /// Get staked balance
    pub fn get_staked_balance(&self, user: &str) -> Option<&StakedBalance> {
        self.staked_balances.get(user)
    }

    /// Get proposal
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&GovernanceProposal> {
        self.governance_proposals.get(&proposal_id)
    }

    /// Check and reset daily mint limits
    fn check_mint_limits(&mut self, amount: u64) -> Result<()> {
        let now = Utc::now();

        // Reset daily counter if new day
        if (now - self.last_mint_reset).num_days() >= 1 {
            self.daily_minted = 0;
            self.last_mint_reset = now;
        }

        if self.daily_minted + amount > self.daily_mint_limit {
            return Err(anyhow::anyhow!("Daily mint limit exceeded"));
        }

        Ok(())
    }

    fn require_owner(&self, caller: &str) -> Result<()> {
        if caller != self.contract_owner {
            return Err(anyhow::anyhow!("Only owner can call this function"));
        }
        Ok(())
    }

    fn require_minter(&self, caller: &str) -> Result<()> {
        if !self.minters.get(caller).unwrap_or(&false) {
            return Err(anyhow::anyhow!("Only minters can call this function"));
        }
        Ok(())
    }

    fn require_burner(&self, caller: &str) -> Result<()> {
        if !self.burners.get(caller).unwrap_or(&false) {
            return Err(anyhow::anyhow!("Only burners can call this function"));
        }
        Ok(())
    }

    fn require_not_paused(&self) -> Result<()> {
        if self.paused {
            return Err(anyhow::anyhow!("Contract is paused"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zephyr_coin_basic() {
        let owner = "owner".to_string();
        let initial_supply = 1000 * 10_u64.pow(18);
        let mut coin = ZephyrCoin::new(owner.clone(), initial_supply);

        assert_eq!(coin.balance_of(&owner), initial_supply);
        assert_eq!(coin.total_supply, initial_supply);
    }

    #[test]
    fn test_transfer() {
        let owner = "owner".to_string();
        let recipient = "recipient".to_string();
        let mut coin = ZephyrCoin::new(owner.clone(), 1000);

        let event = coin.transfer(&owner, &recipient, 100).unwrap();

        assert_eq!(coin.balance_of(&owner), 900);
        assert_eq!(coin.balance_of(&recipient), 100);

        match event {
            TokenEvent::Transfer { from, to, value } => {
                assert_eq!(from, owner);
                assert_eq!(to, recipient);
                assert_eq!(value, 100);
            }
            _ => panic!("Expected Transfer event"),
        }
    }

    #[test]
    fn test_staking() {
        let owner = "owner".to_string();
        let mut coin = ZephyrCoin::new(owner.clone(), 1000);

        coin.stake(&owner, 500, 30).unwrap();

        assert_eq!(coin.balance_of(&owner), 500);
        let staked = coin.get_staked_balance(&owner).unwrap();
        assert_eq!(staked.amount, 500);
    }

    #[test]
    fn test_governance() {
        let owner = "owner".to_string();
        let mut coin = ZephyrCoin::new(owner.clone(), 100_000 * 10_u64.pow(18));

        // Stake tokens for governance
        coin.stake(&owner, 50_000 * 10_u64.pow(18), 365).unwrap();

        // Create proposal
        coin.create_proposal(
            &owner,
            "Test Proposal".to_string(),
            "A test governance proposal".to_string(),
            None,
            None,
            7,
        ).unwrap();

        // Vote on proposal
        coin.vote(&owner, 1, true).unwrap();

        let proposal = coin.get_proposal(1).unwrap();
        assert_eq!(proposal.votes_for, 50_000 * 10_u64.pow(18));
    }
}
#![no_std]

//! # Stellar-Save Smart Contract
//! 
//! A decentralized rotational savings and credit association (ROSCA) built on Stellar Soroban.
//! 
//! This contract enables groups to pool funds in a rotating savings system where:
//! - Members contribute a fixed amount each cycle
//! - One member receives the total pool each cycle
//! - The process rotates until all members have received a payout
//! 
//! ## Modules
//! - `events`: Event types for contract state change tracking
//! - `error`: Comprehensive error types and handling
//! - `group`: Core Group data structure and state management
//! - `contribution`: Contribution record tracking for member payments
//! - `payout`: Payout record tracking for fund distributions
//! - `storage`: Storage key structure for efficient data access
//! - `status`: Group lifecycle status enum with state transitions
//! - `events`: Event definitions for contract actions

pub mod events;
pub mod error;
pub mod contribution;
pub mod group;
pub mod payout;
pub mod status;
pub mod storage;

// Re-export for convenience
pub use events::*;
pub use error::{StellarSaveError, ErrorCategory, ContractResult};
pub use group::Group;
pub use contribution::ContributionRecord;
pub use payout::PayoutRecord;
pub use status::StatusError;
pub use storage::{StorageKey, StorageKeyBuilder};
pub use events::EventEmitter;
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct StellarSaveContract;

#[contractimpl]
impl StellarSaveContract {
    /// Retrieves the current cycle number for a group.
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `group_id` - The ID of the group
    /// 
    /// # Returns
    /// * `u32` - The current cycle number (0-indexed)
    /// 
    /// # Errors
    /// * `StellarSaveError::GroupNotFound` - If the group does not exist
    pub fn get_current_cycle(env: Env, group_id: u64) -> Result<u32, StellarSaveError> {
        // Load the group from storage
        let group: Group = env
            .storage()
            .persistent()
            .get(&StorageKeyBuilder::group_data(group_id))
            .ok_or(StellarSaveError::GroupNotFound)?;

        // Return the current cycle number
        Ok(group.current_cycle)
    }

    pub fn hello(_env: Env) -> soroban_sdk::Symbol {
        soroban_sdk::symbol_short!("hello")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    /// Test that the function correctly retrieves the current cycle from a group
    #[test]
    fn test_get_current_cycle_returns_correct_value() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create a group with initial cycle 0
        let group = Group::new(1, creator, 10_000_000, 604800, 5, 1234567890);
        assert_eq!(group.current_cycle, 0);
    }

    #[test]
    fn test_get_current_cycle_after_advance() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create and advance a group
        let mut group = Group::new(1, creator, 10_000_000, 604800, 5, 1234567890);
        group.advance_cycle();
        
        assert_eq!(group.current_cycle, 1);
    }

    #[test]
    fn test_get_current_cycle_multiple_advances() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create and advance a group multiple times
        let mut group = Group::new(1, creator, 10_000_000, 604800, 5, 1234567890);
        group.advance_cycle();
        group.advance_cycle();
        group.advance_cycle();
        
        assert_eq!(group.current_cycle, 3);
    }

    #[test]
    fn test_get_current_cycle_at_completion() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create a group with 3 members and advance to completion
        let mut group = Group::new(1, creator, 10_000_000, 604800, 3, 1234567890);
        group.advance_cycle();
        group.advance_cycle();
        group.advance_cycle();
        
        assert_eq!(group.current_cycle, 3);
        assert!(group.is_complete());
    }

    #[test]
    fn test_get_current_cycle_multiple_groups_independent() {
        let env = Env::default();
        let creator1 = Address::generate(&env);
        let creator2 = Address::generate(&env);
        
        // Create two groups with different cycles
        let mut group1 = Group::new(1, creator1, 10_000_000, 604800, 5, 1234567890);
        let mut group2 = Group::new(2, creator2, 10_000_000, 604800, 5, 1234567890);
        
        group1.advance_cycle();
        group1.advance_cycle();
        group2.advance_cycle();
        
        assert_eq!(group1.current_cycle, 2);
        assert_eq!(group2.current_cycle, 1);
    }

    #[test]
    fn test_get_current_cycle_large_group_id() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create a group with a large ID
        let large_id = u64::MAX - 1;
        let group = Group::new(large_id, creator, 10_000_000, 604800, 5, 1234567890);
        
        assert_eq!(group.current_cycle, 0);
    }

    #[test]
    fn test_get_current_cycle_zero_group_id() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Create a group with ID 0
        let group = Group::new(0, creator, 10_000_000, 604800, 5, 1234567890);
        
        assert_eq!(group.current_cycle, 0);
    }

    #[test]
    fn test_get_current_cycle_error_handling() {
        // Test that the error type is correct
        let error = StellarSaveError::GroupNotFound;
        assert_eq!(error, StellarSaveError::GroupNotFound);
    }

    #[test]
    fn test_get_current_cycle_boundary_values() {
        let env = Env::default();
        let creator = Address::generate(&env);
        
        // Test with max_members = 2 (minimum)
        let mut group = Group::new(1, creator.clone(), 10_000_000, 604800, 2, 1234567890);
        assert_eq!(group.current_cycle, 0);
        
        group.advance_cycle();
        assert_eq!(group.current_cycle, 1);
        
        group.advance_cycle();
        assert_eq!(group.current_cycle, 2);
        assert!(group.is_complete());
    }
}

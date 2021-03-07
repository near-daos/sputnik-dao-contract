use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet, Vector};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, PromiseOrValue};

use crate::policy::Policy;
use crate::proposals::Proposal;
pub use crate::types::Config;

mod policy;
mod proposals;
mod types;
mod views;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    /// DAO configuration.
    config: Config,
    /// Voting and permissions policy.
    policy: Policy,
    /// Fungible token information and logic.
    token: FungibleToken,
    /// Last available id for the proposals.
    last_proposal_id: u64,
    /// Proposal map from ID to proposal information.
    proposals: LookupMap<u64, Proposal>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(config: Config, policy: Option<Policy>) -> Self {
        assert!(!env::state_exists(), "ERR_CONTRACT_IS_INITIALIZED");
        Self {
            config,
            policy: policy.unwrap_or_default(),
            token: FungibleToken::new(b"t".to_vec()),
            last_proposal_id: 0,
            proposals: LookupMap::new(b"p".to_vec()),
        }
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "".to_string(),
            name: self.config.name.clone(),
            symbol: self.config.symbol.clone(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: self.config.decimals,
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};
    use near_sdk_sim::to_yocto;

    use crate::proposals::{ProposalInput, ProposalKind};

    use super::*;

    fn test_config() -> Config {
        Config {
            name: "Test".to_string(),
            purpose: "to test".to_string(),
            bond: to_yocto("1"),
            symbol: "TEST".to_string(),
            decimals: 24,
        }
    }

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(test_config(), None);
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: test_config(),
            },
        });
        assert_eq!(contract.get_proposal(0).description, "test");
        contract.remove_proposal(0);
    }
}

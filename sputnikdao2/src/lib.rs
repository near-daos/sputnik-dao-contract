use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, PromiseOrValue};

use crate::policy::Policy;
use crate::proposals::{Proposal, ProposalInput, ProposalKind};
pub use crate::types::{Config, Action};

mod policy;
mod proposals;
mod types;
pub mod views;

const KEY_STAGE_CODE: &[u8; 4] = b"CODE";

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

    /// Stages code and creates a proposal for upgrade.
    /// Should attach min of bond and funds to cover the storage of the new contract data.
    #[payable]
    pub fn stage_code(&mut self, #[serializer(borsh)] code: Vec<u8>) {
        assert!(
            !env::storage_has_key(KEY_STAGE_CODE),
            "ERR_CODE_ALREADY_STAGED"
        );
        let proposal = ProposalInput {
            description: format!("Upgrade to {}", hex::encode(env::sha256(&code))),
            kind: ProposalKind::Upgrade,
        };
        self.add_proposal(proposal);
        env::storage_write(KEY_STAGE_CODE, &code);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
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

    use crate::proposals::{ProposalInput, ProposalKind, ProposalStatus};

    use super::*;
    use crate::types::BASE_TOKEN;

    fn test_config() -> Config {
        Config {
            name: "Test".to_string(),
            purpose: "to test".to_string(),
            bond: U128(to_yocto("1")),
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
        contract.act_proposal(0, Action::RemoveProposal);

        let id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Transfer { token_id: BASE_TOKEN.to_string(), receiver_id: accounts(2).into(), amount: to_yocto("100") }
        });
        contract.act_proposal(id, Action::VoteApprove);
        assert_eq!(contract.get_proposal(id).status, ProposalStatus::Approved);
    }
}

use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::metadata::{
    FT_METADATA_SPEC, FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_sdk::{AccountId, env, near_bindgen, PanicOnDefault, Promise, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::serde::{Deserialize, Serialize};

use crate::bounties::{Bounty, BountyClaim};
use crate::policy::Policy;
use crate::proposals::{Proposal, ProposalInput, ProposalKind};
pub use crate::types::{Action, Config};

mod policy;
mod proposals;
mod types;
pub mod views;
mod bounties;

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
    /// Last available id for the bounty.
    last_bounty_id: u64,
    /// Bounties map from ID to bounty information.
    bounties: LookupMap<u64, Bounty>,
    /// Bounty claimers map per user. Allows quickly to query for each users their claims.
    bounty_claimers: LookupMap<AccountId, Vec<BountyClaim>>,
    /// Count of claims per bounty.
    bounty_claims_count: LookupMap<u64, u32>,
    /// Large blob storage.
    blobs: LookupMap<Vec<u8>, AccountId>,
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
            last_bounty_id: 0,
            bounties: LookupMap::new(b"b".to_vec()),
            bounty_claimers: LookupMap::new(b"u".to_vec()),
            bounty_claims_count: LookupMap::new(b"c".to_vec()),
            blobs: LookupMap::new(b"d".to_vec()),
        }
    }

    /// Stores a blob of data under an account paid by the caller.
    #[payable]
    pub fn store_blob(&mut self, #[serializer(borsh)] blob: Vec<u8>) -> Vec<u8> {
        assert!(env::attached_deposit() >= (blob.len() as u128) * env::storage_byte_cost(), "ERR_NOT_ENOUGH_DEPOSIT");
        let hash = env::sha256(&blob);
        self.blobs.insert(&hash, &env::predecessor_account_id());
        env::storage_write(&hash, &blob);
        hash
    }

    /// Remove blob from contract storage and pay back to original storer.
    /// Only original storer can call this.
    pub fn remove_blob(&mut self, hash: Vec<u8>) -> Promise {
        let account_id = self.blobs.remove(&hash).expect("ERR_NO_BLOB");
        assert_eq!(env::predecessor_account_id(), account_id, "ERR_INVALID_CALLER");
        env::storage_remove(&hash);
        let blob = env::storage_get_evicted().unwrap();
        Promise::new(account_id).transfer(env::storage_byte_cost() * (blob.len() as u128))
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
            icon: self.config.icon.clone(),
            reference: self.config.reference.clone(),
            reference_hash: self.config.reference_hash.clone(),
            decimals: self.config.decimals,
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{MockedBlockchain, testing_env};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk_sim::to_yocto;

    use crate::proposals::{ProposalStatus};
    use crate::types::BASE_TOKEN;

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(Config::test_config(), None);
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: Config::test_config(),
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

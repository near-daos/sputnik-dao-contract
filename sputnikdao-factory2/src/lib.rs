mod factory_manager;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8};
use near_sdk::serde_json::{self, json};
use near_sdk::{AccountId, CryptoHash, NearToken, PanicOnDefault, Promise, env, near, require};

use factory_manager::FactoryManager;

type Version = [u8; 2];

// The keys used for writing data to storage via `env::storage_write`.
const DEFAULT_CODE_HASH_KEY: &[u8; 4] = b"CODE";
const FACTORY_OWNER_KEY: &[u8; 5] = b"OWNER";
const CODE_METADATA_KEY: &[u8; 8] = b"METADATA";

// The values used when writing initial data to the storage.
const DAO_CONTRACT_INITIAL_CODE: &[u8] = include_bytes!("../../sputnikdao2/res/sputnikdao2.wasm");
const DAO_CONTRACT_INITIAL_VERSION: Version = [3, 0];
const DAO_CONTRACT_NO_DATA: &str = "no data";

#[near(serializers=[borsh,json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
pub struct DaoContractMetadata {
    // version of the DAO contract code (e.g. [2, 0] -> 2.0, [3, 1] -> 3.1, [4, 0] -> 4.0)
    pub version: Version,
    // commit id of https://github.com/near-daos/sputnik-dao-contract
    // representing a snapshot of the code that generated the wasm
    pub commit_id: String,
    // if available, url to the changelog to see the changes introduced in this version
    pub changelog_url: Option<String>,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct SputnikDAOFactory {
    factory_manager: FactoryManager,
    daos: UnorderedSet<AccountId>,
}

#[near]
impl SputnikDAOFactory {
    #[init]
    pub fn new() -> Self {
        let this = Self {
            factory_manager: FactoryManager {},
            daos: UnorderedSet::new(b"d".to_vec()),
        };
        this.internal_store_initial_contract();
        this
    }

    fn internal_store_initial_contract(&self) {
        self.assert_owner();
        let code = DAO_CONTRACT_INITIAL_CODE.to_vec();
        let sha256_hash = env::sha256_array(&code);
        env::storage_write(&sha256_hash, &code);

        self.store_contract_metadata(
            sha256_hash.into(),
            DaoContractMetadata {
                version: DAO_CONTRACT_INITIAL_VERSION,
                commit_id: String::from(DAO_CONTRACT_NO_DATA),
                changelog_url: None,
            },
            true,
        );
    }

    pub fn set_owner(&self, owner_id: AccountId) {
        self.assert_owner();
        env::storage_write(FACTORY_OWNER_KEY, owner_id.as_bytes());
    }

    pub fn set_default_code_hash(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        let code_hash: CryptoHash = code_hash.into();
        require!(
            env::storage_has_key(&code_hash),
            "Code not found for the given code hash. Please store the code first."
        );
        env::storage_write(DEFAULT_CODE_HASH_KEY, &code_hash);
    }

    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        require!(
            self.get_default_code_hash() != code_hash,
            "Cannot delete the default contract"
        );
        self.factory_manager.delete_contract(code_hash.into());
        self.delete_contract_metadata(code_hash);
    }

    #[payable]
    pub fn create(&mut self, name: AccountId, args: Base64VecU8) {
        let account_id: AccountId = env::current_account_id().sub_account(name).unwrap();
        let callback_args = serde_json::to_vec(&json!({
            "account_id": account_id,
            "attached_deposit": env::attached_deposit(),
            "predecessor_account_id": env::predecessor_account_id()
        }))
        .expect("Failed to serialize");
        self.factory_manager.create_contract(
            self.get_default_code_hash().into(),
            account_id,
            "new",
            &args.0,
            "on_create",
            &callback_args,
        );
    }

    #[private]
    pub fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: NearToken,
        predecessor_account_id: AccountId,
    ) -> bool {
        if near_sdk::is_promise_success() {
            self.daos.insert(&account_id);
            true
        } else {
            Promise::new(predecessor_account_id)
                .transfer(attached_deposit)
                .detach();
            false
        }
    }

    /// Tries to update given account created by this factory to the specified code.
    pub fn update(&self, account_id: AccountId, code_hash: Base58CryptoHash) {
        let caller_id = env::predecessor_account_id();
        require!(
            caller_id == self.get_owner() || caller_id == account_id,
            "Must be updated by the factory owner or the DAO itself"
        );
        require!(
            self.daos.contains(&account_id),
            "Must be contract created by factory"
        );
        self.factory_manager
            .update_contract(account_id, code_hash.into(), "update");
    }

    pub fn get_dao_list(&self) -> Vec<AccountId> {
        self.daos.to_vec()
    }

    /// Get number of created DAOs.
    pub fn get_number_daos(&self) -> u64 {
        self.daos.len()
    }

    /// Get daos in paginated view.
    pub fn get_daos(&self, from_index: u64, limit: u64) -> Vec<AccountId> {
        let elements = self.daos.as_vector();
        (from_index..std::cmp::min(from_index + limit, elements.len()))
            .filter_map(|index| elements.get(index))
            .collect()
    }

    pub fn get_owner(&self) -> AccountId {
        String::from_utf8(
            env::storage_read(FACTORY_OWNER_KEY)
                .unwrap_or(env::current_account_id().as_bytes().to_vec()),
        )
        .expect("INTERNAL_FAIL")
        .parse()
        .unwrap()
    }

    pub fn get_default_code_hash(&self) -> Base58CryptoHash {
        env::storage_read(DEFAULT_CODE_HASH_KEY)
            .and_then(|data| Base58CryptoHash::try_from_slice(&data).ok())
            .expect("Must have code hash")
    }

    pub fn get_default_version(&self) -> Version {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        let default_metadata = deserialized_metadata
            .get(&self.get_default_code_hash().into())
            .expect("INTERNAL_FAIL");
        default_metadata.version
    }

    /// Store new contract. The input are raw bytes of the contract.
    /// Returns base58 of the hash of the contract.
    #[payable]
    pub fn store(&mut self) {
        self.assert_owner();
        let prev_storage = env::storage_usage();
        self.factory_manager.store_contract();
        let storage_cost = env::storage_byte_cost()
            .saturating_mul(env::storage_usage().saturating_sub(prev_storage).into());
        assert!(
            storage_cost <= env::attached_deposit(),
            "Must at least deposit {} to store",
            storage_cost
        );
    }

    /// Returns non serialized code by given code hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        self.factory_manager.get_code(code_hash.into());
    }

    pub fn store_contract_metadata(
        &self,
        code_hash: Base58CryptoHash,
        metadata: DaoContractMetadata,
        set_default: bool,
    ) {
        self.assert_owner();
        let hash: CryptoHash = code_hash.into();
        require!(
            env::storage_has_key(&hash),
            "Code not found for the given code hash. Please store the code first."
        );

        let storage_metadata = env::storage_read(CODE_METADATA_KEY);
        let mut storage_metadata: UnorderedMap<CryptoHash, DaoContractMetadata> =
            if let Some(storage_metadata) = storage_metadata {
                BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL")
            } else {
                UnorderedMap::new(b"m")
            };
        storage_metadata.insert(&hash, &metadata);
        env::storage_write(
            CODE_METADATA_KEY,
            &near_sdk::borsh::to_vec(&storage_metadata).expect("INTERNAL_FAIL"),
        );

        if set_default {
            env::storage_write(DEFAULT_CODE_HASH_KEY, &hash);
        }
    }

    pub fn delete_contract_metadata(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let mut deserialized_metadata: UnorderedMap<CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        deserialized_metadata.remove(&code_hash.into());
        env::storage_write(
            CODE_METADATA_KEY,
            &near_sdk::borsh::to_vec(&deserialized_metadata).expect("INTERNAL_FAIL"),
        );
    }

    pub fn get_contracts_metadata(&self) -> Vec<(Base58CryptoHash, DaoContractMetadata)> {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        deserialized_metadata.to_vec()
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.get_owner(),
            env::predecessor_account_id(),
            "Must be owner"
        );
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{PromiseResult, RuntimeFeesConfig, test_vm_config, testing_env};

    use near_api::NearToken;

    use super::*;

    #[test]
    #[should_panic(expected = "ERR_NOT_ENOUGH_DEPOSIT")]
    fn test_create_error() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(accounts(0))
                .predecessor_account_id(accounts(0))
                .build()
        );
        let mut factory = SputnikDAOFactory::new();

        testing_env!(
            context
                .attached_deposit(NearToken::from_millinear(1))
                .build()
        );
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());
    }

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(accounts(0))
                .predecessor_account_id(accounts(0))
                .build()
        );
        let mut factory = SputnikDAOFactory::new();

        testing_env!(
            context
                .attached_deposit(NearToken::from_millinear(10))
                .build()
        );
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());

        testing_env!(
            context.predecessor_account_id(accounts(0)).build(),
            test_vm_config(),
            RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        factory.on_create(
            format!("test.{}", accounts(0)).parse().unwrap(),
            NearToken::from_millinear(10),
            accounts(0),
        );
        assert_eq!(
            factory.get_dao_list(),
            vec![format!("test.{}", accounts(0).as_str())]
        );
        assert_eq!(
            factory.get_daos(0, 100),
            vec![format!("test.{}", accounts(0).as_str())]
        );
    }

    //              #################################              //
    //              #    Factory ownership tests    #              //
    //              #################################              //

    #[test]
    fn test_factory_can_get_current_owner() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(alice())
                .predecessor_account_id(alice())
                .build()
        );
        let factory = SputnikDAOFactory::new();

        testing_env!(context.is_view(true).build());
        assert_eq!(factory.get_owner(), alice());
    }

    #[test]
    #[should_panic]
    fn test_factory_fails_setting_owner_from_not_owner_account() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(alice())
                .predecessor_account_id(alice())
                .build()
        );
        let factory = SputnikDAOFactory::new();

        testing_env!(
            context
                .current_account_id(alice())
                .predecessor_account_id(carol())
                .build()
        );
        factory.set_owner(bob());
    }

    #[test]
    fn test_owner_can_be_a_dao_account() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(bob())
                .predecessor_account_id(bob())
                .attached_deposit(NearToken::from_near(6))
                .build()
        );
        let mut factory = SputnikDAOFactory::new();

        factory.create(bob(), "{}".as_bytes().to_vec().into());

        factory.set_owner("bob.sputnik-dao.near".parse().unwrap());

        assert_eq!(factory.get_owner().as_str(), "bob.sputnik-dao.near")
    }

    #[test]
    fn test_owner_gets_succesfully_updated() {
        let mut context = VMContextBuilder::new();
        testing_env!(
            context
                .current_account_id(accounts(0))
                .predecessor_account_id(accounts(0))
                .attached_deposit(NearToken::from_near(5))
                .build()
        );
        let factory = SputnikDAOFactory::new();

        assert_ne!(factory.get_owner(), bob());

        factory.set_owner(bob());

        assert_eq!(factory.get_owner(), bob())
    }
}

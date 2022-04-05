mod factory_manager;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{self, json};
use near_sdk::{env, near_bindgen, AccountId, Balance, CryptoHash, Gas, PanicOnDefault, Promise};

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

// Gas & Costs for blob storage
const GAS_STORE_CONTRACT_LEFTOVER: Gas = Gas(20_000_000_000_000);
const ON_REMOVE_CONTRACT_GAS: Gas = Gas(10_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoContractMetadata {
    // version of the DAO contract code (e.g. [2, 0] -> 2.0, [3, 1] -> 3.1, [4, 0] -> 4.0)
    pub version: Version,
    // commit id of https://github.com/near-daos/sputnik-dao-contract
    // representing a snapshot of the code that generated the wasm
    pub commit_id: String,
    // if available, url to the changelog to see the changes introduced in this version
    pub changelog_url: Option<String>,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct SputnikDAOFactory {
    factory_manager: FactoryManager,
    daos: UnorderedSet<AccountId>,
}

#[near_bindgen]
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
        let sha256_hash = env::sha256(&code);
        env::storage_write(&sha256_hash, &code);

        self.store_contract_metadata(
            slice_to_hash(&sha256_hash),
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
        assert!(
            env::storage_has_key(&code_hash),
            "Code not found for the given code hash. Please store the code first."
        );
        env::storage_write(DEFAULT_CODE_HASH_KEY, &code_hash);
    }

    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        self.factory_manager.delete_contract(code_hash);
        self.delete_contract_metadata(code_hash);
    }

    #[payable]
    pub fn create(&mut self, name: AccountId, args: Base64VecU8) {
        let account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();
        let callback_args = serde_json::to_vec(&json!({
            "account_id": account_id,
            "attached_deposit": U128(env::attached_deposit()),
            "predecessor_account_id": env::predecessor_account_id()
        }))
        .expect("Failed to serialize");
        self.factory_manager.create_contract(
            self.get_default_code_hash(),
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
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool {
        if near_sdk::is_promise_success() {
            self.daos.insert(&account_id);
            true
        } else {
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            false
        }
    }

    /// Tries to update given account created by this factory to the specified code.
    pub fn update(&self, account_id: AccountId, code_hash: Base58CryptoHash) {
        let caller_id = env::predecessor_account_id();
        assert!(
            caller_id == self.get_owner() || caller_id == account_id,
            "Must be updated by the factory owner or the DAO itself"
        );
        assert!(
            self.daos.contains(&account_id),
            "Must be contract created by factory"
        );
        self.factory_manager
            .update_contract(account_id, code_hash, "update");
    }

    /// Allows a DAO to store the official factory version as a blob, funded by the DAO wanting to upgrade
    /// Required to successfully upgrade a DAO via proposals (proposal to store blob, proposal to upgrade from local blob)
    /// Only intended for sputnik v2 DAO's created by sputnik factory
    /// Payment is needed to cover storage costs for code blob size, paid by the DAO and returned upon blob removal
    #[payable]
    pub fn store_contract_self(&mut self, code_hash: Base58CryptoHash) {
        let account_id = env::predecessor_account_id();
        let method_name = "store_blob";

        let hash: CryptoHash = code_hash.into();
        assert!(
            env::storage_has_key(&hash),
            "Code not found for the given code hash. Please store the code first."
        );

        // Lock down contract upgrades to this factory:
        let dao_id = env::predecessor_account_id().to_string();
        let idx = dao_id.find('.').expect("INTERNAL_FAIL");
        // ex: sputnik-dao.near
        let factory_id = &dao_id[idx + 1..];

        assert_eq!(
            factory_id,
            env::current_account_id().as_str(),
            "Wrong factory"
        );

        let dao_contract_code = env::storage_read(&hash).expect("CODE_HASH_NONEXIST");

        // Compute and use the correct amount needed for storage
        let blob_len = dao_contract_code.len();
        let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();

        // Confirm payment before proceeding
        assert!(
            storage_cost <= env::attached_deposit(),
            "Must at least deposit {} to store",
            storage_cost
        );

        // refund the extra cost
        let extra_attached_deposit = env::attached_deposit() - storage_cost;
        Promise::new(account_id.clone()).transfer(extra_attached_deposit);

        // Create a promise toward given account.
        let promise_id = env::promise_batch_create(&account_id);
        env::promise_batch_action_function_call(
            promise_id,
            method_name,
            &dao_contract_code,
            storage_cost,
            env::prepaid_gas() - env::used_gas() - GAS_STORE_CONTRACT_LEFTOVER,
        );
        env::promise_return(promise_id);
    }

    /// Allows a DAO to remove the blob stored in its DAO storage, and reclaim the storage cost
    pub fn remove_contract_self(&mut self, code_hash: Base58CryptoHash) {
        let account_id = env::predecessor_account_id();
        let factory_id = env::current_account_id();
        let method_name = "remove_blob";

        // NOTE: Not verifing the hash, in case factory removes a hash before DAO does
        let method_args = &json!({ "hash": &code_hash }).to_string().into_bytes();
        let callback_method = "on_remove_contract_self";
        let callback_args = &json!({ "account_id": &account_id, "code_hash": &code_hash })
            .to_string()
            .into_bytes();

        // Create a promise toward given account.
        let promise_id = env::promise_batch_create(&account_id);
        env::promise_batch_action_function_call(
            promise_id,
            method_name,
            method_args,
            NO_DEPOSIT,
            GAS_STORE_CONTRACT_LEFTOVER,
        );
        // attach callback to the factory.
        let _ = env::promise_then(
            promise_id,
            factory_id,
            callback_method,
            callback_args,
            NO_DEPOSIT,
            ON_REMOVE_CONTRACT_GAS,
        );
        env::promise_return(promise_id);
    }

    /// Upon blob remove, compute the balance (if any) that got paid to the factory,
    /// since it was the "owner" of the blob stored on the DAO.
    /// Send this balance back to the DAO, since it was the original funder
    #[private]
    pub fn on_remove_contract_self(
        &mut self,
        account_id: AccountId,
        code_hash: Base58CryptoHash,
    ) -> bool {
        if near_sdk::is_promise_success() {
            // Compute the actual storage cost for an accurate refund
            let hash: CryptoHash = code_hash.into();
            let dao_contract_code = env::storage_read(&hash).expect("CODE_HASH_NONEXIST");
            let blob_len = dao_contract_code.len();
            let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();
            Promise::new(account_id).transfer(storage_cost);
            true
        } else {
            false
        }
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
        AccountId::new_unchecked(
            String::from_utf8(
                env::storage_read(FACTORY_OWNER_KEY)
                    .unwrap_or(env::current_account_id().as_bytes().to_vec()),
            )
            .expect("INTERNAL_FAIL"),
        )
    }

    pub fn get_default_code_hash(&self) -> Base58CryptoHash {
        slice_to_hash(&env::storage_read(DEFAULT_CODE_HASH_KEY).expect("Must have code hash"))
    }

    pub fn get_default_version(&self) -> Version {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        let default_metadata = deserialized_metadata
            .get(&self.get_default_code_hash())
            .expect("INTERNAL_FAIL");
        default_metadata.version
    }

    /// Returns non serialized code by given code hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        self.factory_manager.get_code(code_hash);
    }

    pub fn store_contract_metadata(
        &self,
        code_hash: Base58CryptoHash,
        metadata: DaoContractMetadata,
        set_default: bool,
    ) {
        self.assert_owner();
        let hash: CryptoHash = code_hash.into();
        assert!(
            env::storage_has_key(&hash),
            "Code not found for the given code hash. Please store the code first."
        );

        let storage_metadata = env::storage_read(CODE_METADATA_KEY);
        if storage_metadata.is_none() {
            let mut storage_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
                UnorderedMap::new(b"m".to_vec());
            storage_metadata.insert(&code_hash, &metadata);
            let serialized_metadata =
                BorshSerialize::try_to_vec(&storage_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        } else {
            let storage_metadata = storage_metadata.expect("INTERNAL_FAIL");
            let mut deserialized_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
                BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
            deserialized_metadata.insert(&code_hash, &metadata);
            let serialized_metadata =
                BorshSerialize::try_to_vec(&deserialized_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        }

        if set_default {
            env::storage_write(DEFAULT_CODE_HASH_KEY, &hash);
        }
    }

    pub fn delete_contract_metadata(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let mut deserialized_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        deserialized_metadata.remove(&code_hash);
        let serialized_metadata =
            BorshSerialize::try_to_vec(&deserialized_metadata).expect("INTERNAL_FAIL");
        env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
    }

    pub fn get_contracts_metadata(&self) -> Vec<(Base58CryptoHash, DaoContractMetadata)> {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: UnorderedMap<Base58CryptoHash, DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        return deserialized_metadata.to_vec();
    }

    fn assert_owner(&self) {
        assert_eq!(
            self.get_owner(),
            env::predecessor_account_id(),
            "Must be owner"
        );
    }
}

pub fn slice_to_hash(hash: &[u8]) -> Base58CryptoHash {
    let mut result: CryptoHash = [0; 32];
    result.copy_from_slice(&hash);
    Base58CryptoHash::from(result)
}

/// Store new contract. Non serialized argument is the contract.
/// Returns base58 of the hash of the contract.
#[no_mangle]
pub extern "C" fn store() {
    env::setup_panic_hook();
    let contract: SputnikDAOFactory = env::state_read().expect("Contract is not initialized");
    contract.assert_owner();
    let prev_storage = env::storage_usage();
    contract.factory_manager.store_contract();
    let storage_cost = (env::storage_usage() - prev_storage) as u128 * env::storage_byte_cost();
    assert!(
        storage_cost <= env::attached_deposit(),
        "Must at least deposit {} to store",
        storage_cost
    );
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, PromiseResult};

    use near_sdk_sim::to_yocto;

    use super::*;

    #[test]
    #[should_panic(expected = "ERR_NOT_ENOUGH_DEPOSIT")]
    fn test_create_error() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());
        let mut factory = SputnikDAOFactory::new();

        testing_env!(context.attached_deposit(to_yocto("5")).build());
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());
    }

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());
        let mut factory = SputnikDAOFactory::new();

        testing_env!(context.attached_deposit(to_yocto("6")).build());
        factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());

        testing_env!(
            context.predecessor_account_id(accounts(0)).build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        factory.on_create(
            format!("test.{}", accounts(0)).parse().unwrap(),
            U128(to_yocto("6")),
            accounts(0),
        );
        assert_eq!(
            factory.get_dao_list(),
            vec![format!("test.{}", accounts(0)).parse().unwrap()]
        );
        assert_eq!(
            factory.get_daos(0, 100),
            vec![format!("test.{}", accounts(0)).parse().unwrap()]
        );
    }

    //              #################################              //
    //              #    Factory ownership tests    #              //
    //              #################################              //

    #[test]
    fn test_factory_can_get_current_owner() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(alice())
            .predecessor_account_id(alice())
            .attached_deposit(to_yocto("5"))
            .build());
        let factory = SputnikDAOFactory::new();

        assert_eq!(factory.get_owner(), alice());
    }

    #[test]
    #[should_panic]
    fn test_factory_fails_setting_owner_from_not_owner_account() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(alice())
            .predecessor_account_id(carol())
            .attached_deposit(to_yocto("5"))
            .build());
        let factory = SputnikDAOFactory::new();

        factory.set_owner(bob());
    }

    #[test]
    fn test_owner_can_be_a_dao_account() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(bob())
            .predecessor_account_id(bob())
            .attached_deposit(to_yocto("6"))
            .build());
        let mut factory = SputnikDAOFactory::new();

        factory.create(bob(), "{}".as_bytes().to_vec().into());

        factory.set_owner(AccountId::new_unchecked("bob.sputnik-dao.near".to_string()));

        assert_eq!(
            factory.get_owner(),
            AccountId::new_unchecked("bob.sputnik-dao.near".to_string())
        )
    }

    #[test]
    fn test_owner_gets_succesfully_updated() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .attached_deposit(to_yocto("5"))
            .build());
        let factory = SputnikDAOFactory::new();

        assert_ne!(factory.get_owner(), bob());

        factory.set_owner(bob());

        assert_eq!(factory.get_owner(), bob())
    }
}

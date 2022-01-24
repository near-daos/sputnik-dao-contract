mod factory_manager;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{self, json};
use near_sdk::{env, near_bindgen, AccountId, CryptoHash, PanicOnDefault, Promise};

use factory_manager::FactoryManager;

// The keys used for writing data to storage via `env::storage_write`.
const LATEST_CODE_HASH_KEY: &[u8; 4] = b"CODE";
const FACTORY_OWNER_KEY: &[u8; 5] = b"OWNER";
const CODE_METADATA_KEY: &[u8; 8] = b"METADATA";

const DAO_CONTRACT_CODE: &[u8] = include_bytes!("../../sputnikdao2/res/sputnikdao2.wasm");

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoContractMetadata {
    pub code_hash: Base58CryptoHash,
    pub version: String,
    pub commit_id: String,
    pub readme: String,
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
        this.internal_store_initial_version();
        this
    }

    fn internal_store_initial_version(&self) {
        let code = DAO_CONTRACT_CODE.to_vec();
        let sha256_hash = env::sha256(&code);
        env::storage_write(&sha256_hash, &code);
    }

    pub fn set_owner(&self, owner_id: AccountId) {
        self.assert_owner();
        env::storage_write(FACTORY_OWNER_KEY, owner_id.as_bytes());
    }

    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        self.assert_owner();
        self.factory_manager.delete_contract(code_hash);
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
            self.get_latest_code_hash(),
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

    /// Tries to upgrade given account created by this factory to the latest code.
    pub fn upgrade(&self, account_id: AccountId) {
        self.assert_owner();
        assert!(
            self.daos.contains(&account_id),
            "Must be contract created by factory"
        );
        self.factory_manager
            .update_contract(account_id, self.get_latest_code_hash(), "update");
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

    pub fn get_latest_code_hash(&self) -> Base58CryptoHash {
        slice_to_hash(&env::storage_read(LATEST_CODE_HASH_KEY).expect("Must have code hash"))
    }

    /// Returns non serialized code by given code hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        self.factory_manager.get_code(code_hash);
    }

    pub fn store_contract_metadata(&self, metadata: DaoContractMetadata) {
        self.assert_owner();
        let code_hash: CryptoHash = metadata.code_hash.into();
        let storage_metadata = env::storage_read(CODE_METADATA_KEY);

        if storage_metadata.is_none() {
            let storage_metadata = vec![metadata];
            let serialized_metadata =
                BorshSerialize::try_to_vec(&storage_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        } else {
            let storage_metadata = storage_metadata.expect("INTERNAL_FAIL");
            let mut deserialized_metadata: Vec<DaoContractMetadata> =
                BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
            deserialized_metadata.push(metadata);
            let serialized_metadata =
                BorshSerialize::try_to_vec(&deserialized_metadata).expect("INTERNAL_FAIL");
            env::storage_write(CODE_METADATA_KEY, &serialized_metadata);
        }

        env::storage_write(LATEST_CODE_HASH_KEY, &code_hash);
    }

    pub fn get_contract_metadata(&self) -> Vec<DaoContractMetadata> {
        let storage_metadata = env::storage_read(CODE_METADATA_KEY).expect("INTERNAL_FAIL");
        let deserialized_metadata: Vec<DaoContractMetadata> =
            BorshDeserialize::try_from_slice(&storage_metadata).expect("INTERNAL_FAIL");
        return deserialized_metadata;
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
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, PromiseResult};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .current_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());
        let mut factory = SputnikDAOFactory::new();

        testing_env!(context.attached_deposit(10).build());
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
            U128(10),
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
}

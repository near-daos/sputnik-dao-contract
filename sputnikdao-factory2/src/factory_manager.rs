//! Module for standard generic contract factory manager.
//! TODO: move to near-sdk standards library.

use near_sdk::json_types::Base58CryptoHash;
use near_sdk::{AccountId, CryptoHash, Gas, NearToken, env, near, require, serde_json};

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: Gas = Gas::from_tgas(40);

/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: Gas = Gas::from_tgas(10);

const NO_DEPOSIT: NearToken = NearToken::from_near(0);

/// Factory manager that allows to store/load contracts by hash directly in the storage.
/// Uses directly underlying host functions to not load any of the data into WASM memory.
#[near(serializers=[borsh])]
pub struct FactoryManager {}

impl FactoryManager {
    /// Store contract from input.
    pub fn store_contract(&self) {
        let input = env::input().expect("ERR_NO_INPUT");
        let sha256_hash = env::sha256_array(&input);
        require!(!env::storage_has_key(&sha256_hash), "ERR_ALREADY_EXISTS");
        env::storage_write(&sha256_hash, &input);

        let blob_hash_str = serde_json::to_vec(&Base58CryptoHash::from(sha256_hash)).unwrap();
        env::value_return(&blob_hash_str);
    }

    /// Delete code from the contract.
    pub fn delete_contract(&self, code_hash: CryptoHash) {
        env::storage_remove(&code_hash);
    }

    /// Get code for given hash.
    pub fn get_code(&self, code_hash: CryptoHash) {
        // Load the hash from storage.
        let code = env::storage_read(&code_hash).expect("ERR_NO_HASH");
        // Return as value.
        env::value_return(&code);
    }

    /// Forces update on the given contract.
    /// Contract must support update by factory for this via permission check.
    pub fn update_contract(
        &self,
        dao_account_id: AccountId,
        target_code_hash: CryptoHash,
        update_method_name: &str,
    ) {
        // Load the hash from storage.
        let code = env::storage_read(&target_code_hash).expect("ERR_NO_HASH");
        // Create a promise toward given account.
        let promise_id = env::promise_batch_create(&dao_account_id);
        // Call `update` method, which should also handle migrations.
        env::promise_batch_action_function_call_weight(
            promise_id,
            update_method_name,
            &code,
            NO_DEPOSIT,
            Gas::from_gas(0),
            near_sdk::GasWeight(1),
        );
        env::promise_return(promise_id);
    }

    /// Create given contract with args and callback factory.
    pub fn create_contract(
        &self,
        code_hash: CryptoHash,
        account_id: AccountId,
        new_method: &str,
        args: &[u8],
        callback_method: &str,
        callback_args: &[u8],
    ) {
        let attached_deposit = env::attached_deposit();
        let factory_account_id = env::current_account_id();
        // Minimal deposit is required to cover the account registration and DAO config storage
        const MINIMAL_DEPOSIT: NearToken = NearToken::from_millinear(10);
        assert!(
            attached_deposit >= MINIMAL_DEPOSIT,
            "ERR_NOT_ENOUGH_DEPOSIT:{}",
            MINIMAL_DEPOSIT.as_yoctonear(),
        );
        // Check that such contract exists.
        require!(env::storage_has_key(&code_hash), "Contract doesn't exist");
        // Schedule a Promise tx to account_id.
        let promise_id = env::promise_batch_create(&account_id);
        // Create account first.
        env::promise_batch_action_create_account(promise_id);
        // Transfer attached deposit.
        env::promise_batch_action_transfer(promise_id, attached_deposit);
        // Deploy contract.
        env::promise_batch_action_use_global_contract(promise_id, &code_hash);
        // call `new` with given arguments.
        env::promise_batch_action_function_call(
            promise_id,
            new_method,
            args,
            NO_DEPOSIT,
            CREATE_CALL_GAS,
        );
        // attach callback to the factory.
        let _ = env::promise_then(
            promise_id,
            factory_account_id,
            callback_method,
            callback_args,
            NO_DEPOSIT,
            ON_CREATE_CALL_GAS,
        );
        env::promise_return(promise_id);
    }
}

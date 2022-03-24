//! Module for standard generic contract factory manager.
//! TODO: move to near-sdk standards library.

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json;
use near_sdk::{env, sys, AccountId, Balance, CryptoHash, Gas};

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: Gas = Gas(10_000_000_000_000);

/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: Gas = Gas(10_000_000_000_000);

/// Leftover gas after creating promise and calling update.
const GAS_UPDATE_LEFTOVER: Gas = Gas(10_000_000_000_000);

const NO_DEPOSIT: Balance = 0;

/// Factory manager that allows to store/load contracts by hash directly in the storage.
/// Uses directly underlying host functions to not load any of the data into WASM memory.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct FactoryManager {}

impl FactoryManager {
    /// Store contract from input.
    pub fn store_contract(&self) {
        unsafe {
            // Load input into register 0.
            sys::input(0);
            // Compute sha256 hash of register 0 and store in register 1.
            sys::sha256(u64::MAX as _, 0 as _, 1);
            // Check if such blob is already stored.
            assert_eq!(
                sys::storage_has_key(u64::MAX as _, 1 as _),
                0,
                "ERR_ALREADY_EXISTS"
            );
            // Store key-value pair. The key is represented by register 1 and the value by register 0.
            sys::storage_write(u64::MAX as _, 1 as _, u64::MAX as _, 0 as _, 2);
            // Load register 1 into blob_hash.
            let blob_hash = [0u8; 32];
            sys::read_register(1, blob_hash.as_ptr() as _);
            // Return from function value of register 1.
            let blob_hash_str = serde_json::to_string(&Base58CryptoHash::from(blob_hash))
                .unwrap()
                .into_bytes();
            sys::value_return(blob_hash_str.len() as _, blob_hash_str.as_ptr() as _);
        }
    }

    /// Delete code from the contract.
    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        let code_hash: CryptoHash = code_hash.into();
        env::storage_remove(&code_hash);
    }

    /// Get code for given hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        let code_hash: CryptoHash = code_hash.into();
        unsafe {
            // Check that such contract exists.
            assert!(env::storage_has_key(&code_hash), "Contract doesn't exist");
            // Load the hash from storage.
            sys::storage_read(code_hash.len() as _, code_hash.as_ptr() as _, 0);
            // Return as value.
            sys::value_return(u64::MAX as _, 0 as _);
        }
    }

    /// Forces update on the given contract.
    /// Contract must support update by factory for this via permission check.
    pub fn update_contract(
        &self,
        account_id: AccountId,
        code_hash: Base58CryptoHash,
        method_name: &str,
    ) {
        let code_hash: CryptoHash = code_hash.into();
        let account_id = account_id.as_bytes().to_vec();
        unsafe {
            // Check that such contract exists.
            assert!(env::storage_has_key(&code_hash), "Contract doesn't exist");
            // Load the hash from storage.
            sys::storage_read(code_hash.len() as _, code_hash.as_ptr() as _, 0);
            // Create a promise toward given account.
            let promise_id =
                sys::promise_batch_create(account_id.len() as _, account_id.as_ptr() as _);
            // Call `update` method, which should also handle migrations.
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                u64::MAX as _,
                0,
                &NO_DEPOSIT as *const u128 as _,
                (env::prepaid_gas() - env::used_gas() - GAS_UPDATE_LEFTOVER).0,
            );
            sys::promise_return(promise_id);
        }
    }

    /// Create given contract with args and callback factory.
    pub fn create_contract(
        &self,
        code_hash: Base58CryptoHash,
        account_id: AccountId,
        new_method: &str,
        args: &[u8],
        callback_method: &str,
        callback_args: &[u8],
    ) {
        let code_hash: CryptoHash = code_hash.into();
        let attached_deposit = env::attached_deposit();
        let factory_account_id = env::current_account_id().as_bytes().to_vec();
        let account_id = account_id.as_bytes().to_vec();
        unsafe {
            // Check that such contract exists.
            assert_eq!(
                sys::storage_has_key(code_hash.len() as _, code_hash.as_ptr() as _),
                1,
                "Contract doesn't exist"
            );
            // Load input (wasm code) into register 0.
            sys::storage_read(code_hash.len() as _, code_hash.as_ptr() as _, 0);
            // schedule a Promise tx to account_id
            let promise_id =
                sys::promise_batch_create(account_id.len() as _, account_id.as_ptr() as _);
            // create account first.
            sys::promise_batch_action_create_account(promise_id);
            // transfer attached deposit.
            sys::promise_batch_action_transfer(promise_id, &attached_deposit as *const u128 as _);
            // deploy contract (code is taken from register 0).
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // call `new` with given arguments.
            sys::promise_batch_action_function_call(
                promise_id,
                new_method.len() as _,
                new_method.as_ptr() as _,
                args.len() as _,
                args.as_ptr() as _,
                &NO_DEPOSIT as *const u128 as _,
                CREATE_CALL_GAS.0,
            );
            // attach callback to the factory.
            let _ = sys::promise_then(
                promise_id,
                factory_account_id.len() as _,
                factory_account_id.as_ptr() as _,
                callback_method.len() as _,
                callback_method.as_ptr() as _,
                callback_args.len() as _,
                callback_args.as_ptr() as _,
                &NO_DEPOSIT as *const u128 as _,
                ON_CREATE_CALL_GAS.0,
            );
            sys::promise_return(promise_id);
        }
    }
}

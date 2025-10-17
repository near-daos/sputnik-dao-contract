//! Module for standard generic contract factory manager.
//! TODO: move to near-sdk standards library.

use near_sdk::json_types::Base58CryptoHash;
use near_sdk::{env, near, serde_json, AccountId, CryptoHash, Gas, NearToken};

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
        let sha256_hash = env::sha256(&input);
        assert!(!env::storage_has_key(&sha256_hash), "ERR_ALREADY_EXISTS");
        env::storage_write(&sha256_hash, &input);

        let mut blob_hash = [0u8; 32];
        blob_hash.copy_from_slice(&sha256_hash);
        let blob_hash_str = serde_json::to_string(&Base58CryptoHash::from(blob_hash))
            .unwrap()
            .into_bytes();
        env::value_return(&blob_hash_str);
    }

    /// Delete code from the contract.
    pub fn delete_contract(&self, code_hash: Base58CryptoHash) {
        let code_hash: CryptoHash = code_hash.into();
        env::storage_remove(&code_hash);
    }

    /// Get code for given hash.
    pub fn get_code(&self, code_hash: Base58CryptoHash) {
        let code_hash: CryptoHash = code_hash.into();
        // Check that such contract exists.
        assert!(env::storage_has_key(&code_hash), "Contract doesn't exist");
        // Load the hash from storage.
        let code = env::storage_read(&code_hash).unwrap();
        // Return as value.
        env::value_return(&code);
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
        // Check that such contract exists.
        assert!(env::storage_has_key(&code_hash), "Contract doesn't exist");
        // Load the hash from storage.
        let code = env::storage_read(&code_hash).expect("ERR_NO_HASH");
        // Create a promise toward given account.
        let promise_id = env::promise_batch_create(&account_id);
        // Call `update` method, which should also handle migrations.
        env::promise_batch_action_function_call_weight(
            promise_id,
            method_name,
            &code,
            NO_DEPOSIT,
            Gas::from_gas(0),
            near_sdk::GasWeight::default(),
        );
        env::promise_return(promise_id);
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
        let factory_account_id = env::current_account_id();
        // Check that such contract exists.
        assert!(env::storage_has_key(&code_hash), "Contract doesn't exist");
        // Load input (wasm code).
        let code = env::storage_read(&code_hash).expect("ERR_NO_HASH");
        // Compute storage cost.
        let code_len = code.len();
        let storage_cost = env::storage_byte_cost().saturating_mul((code_len + 32) as u128);
        assert!(
            attached_deposit >= storage_cost,
            "ERR_NOT_ENOUGH_DEPOSIT:{}",
            storage_cost
        );
        // Schedule a Promise tx to account_id.
        let promise_id = env::promise_batch_create(&account_id);
        // Create account first.
        env::promise_batch_action_create_account(promise_id);
        // Transfer attached deposit.
        env::promise_batch_action_transfer(promise_id, attached_deposit);
        // Deploy contract.
        env::promise_batch_action_deploy_contract(promise_id, &code);
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

    /// Create DAO instance from global contract using code hash.
    /// This method uses a global contract to reduce storage costs.
    /// The code_hash should be the hash of the deployed global contract.
    pub fn create_global_contract(
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
        let factory_account_id = env::current_account_id();

        // Calculate gas cost to deduct from deposit
        // The factory pays for CREATE_CALL_GAS but any unused gas refund goes to the DAO
        // To prevent the factory from losing funds, we deduct the maximum gas cost from the transfer
        // Gas price is approximately 100,000,000 yoctoNEAR per gas unit
        // For 40 TGas: 40 * 10^12 gas * 10^8 yoctoNEAR/gas = 4 * 10^21 yoctoNEAR (~0.004 NEAR)
        // We use a conservative estimate of 0.01 NEAR to cover gas costs and variations in gas price
        let gas_cost_buffer = NearToken::from_millinear(10); // 0.01 NEAR

        // Ensure we have enough deposit to cover the gas cost buffer
        assert!(
            attached_deposit > gas_cost_buffer,
            "Attached deposit must be greater than gas cost buffer of {} yoctoNEAR",
            gas_cost_buffer.as_yoctonear()
        );

        // Transfer deposit minus gas cost buffer; unused gas will be refunded to the DAO
        let transfer_amount = attached_deposit.saturating_sub(gas_cost_buffer);

        // Create the subaccount and deploy global contract by hash
        let promise_id = env::promise_batch_create(&account_id);
        // Create account first
        env::promise_batch_action_create_account(promise_id);
        // Transfer deposit minus gas cost (unused gas will be refunded to DAO)
        env::promise_batch_action_transfer(promise_id, transfer_amount);
        // Deploy contract using global contract hash
        env::promise_batch_action_use_global_contract(promise_id, &code_hash);
        // Call initialization method
        env::promise_batch_action_function_call(
            promise_id,
            new_method,
            args,
            NO_DEPOSIT,
            CREATE_CALL_GAS,
        );

        // Attach callback to the factory
        let callback_promise = env::promise_then(
            promise_id,
            factory_account_id,
            callback_method,
            callback_args,
            NO_DEPOSIT,
            ON_CREATE_CALL_GAS,
        );
        env::promise_return(callback_promise);
    }
}

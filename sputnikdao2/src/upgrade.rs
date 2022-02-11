//! Logic to upgrade Sputnik contracts.

use near_sdk::serde_json::json;
use near_sdk::Gas;

use crate::*;

const FACTORY_KEY: &[u8; 7] = b"FACTORY";
const ERR_MUST_BE_SELF_OR_FACTORY: &str = "ERR_MUST_BE_SELF_OR_FACTORY";
const SELF_MIGRATE_METHOD_NAME: &[u8; 7] = b"migrate";
const UPDATE_GAS_LEFTOVER: Gas = Gas(5_000_000_000_000);
const FACTORY_UPDATE_GAS_LEFTOVER: Gas = Gas(15_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

/// Gas for upgrading this contract on promise creation + deploying new contract.
pub const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas(30_000_000_000_000);

pub const GAS_FOR_UPGRADE_REMOTE_DEPLOY: Gas = Gas(10_000_000_000_000);

/// Info about factory that deployed this contract and if auto-update is allowed.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct FactoryInfo {
    pub factory_id: AccountId,
    pub auto_update: bool,
}

pub fn get_default_factory_id() -> AccountId {
    // ex: mydao.sputnik-dao.near
    let dao_id = env::current_account_id().to_string();
    let idx = dao_id.find('.').expect("INTERNAL_FAIL");
    // ex: sputnik-dao.near
    let factory_id = &dao_id[idx + 1..];

    AccountId::new_unchecked(String::from(factory_id))
}

/// Fetches factory info from the storage.
/// By design not using contract STATE to allow for upgrade of stuck contracts from factory.
pub(crate) fn internal_get_factory_info() -> FactoryInfo {
    env::storage_read(FACTORY_KEY)
        .map(|value| FactoryInfo::try_from_slice(&value).expect("INTERNAL_FAIL"))
        .unwrap_or_else(|| FactoryInfo {
            factory_id: get_default_factory_id(),
            auto_update: true,
        })
}

pub(crate) fn internal_set_factory_info(factory_info: &FactoryInfo) {
    env::storage_write(
        FACTORY_KEY,
        &factory_info.try_to_vec().expect("INTERNAL_FAIL"),
    );
}

/// Function that receives new contract, updates and calls migration.
/// Two options who call it:
///  - current account, in case of fetching contract code from factory;
///  - factory, if this contract allows to factory-update;
#[no_mangle]
pub fn update() {
    env::setup_panic_hook();
    let factory_info = internal_get_factory_info();
    let current_id = env::current_account_id();
    assert!(
        env::predecessor_account_id() == current_id
            || (env::predecessor_account_id() == factory_info.factory_id
                && factory_info.auto_update),
        "{}",
        ERR_MUST_BE_SELF_OR_FACTORY
    );
    let is_callback = env::predecessor_account_id() == current_id;
    unsafe {
        // Load code into register 0 result from the input argument if factory call or from promise if callback.
        if is_callback {
            sys::promise_result(0, 0);
        } else {
            sys::input(0);
        }
        // Update current contract with code from register 0.
        let promise_id = sys::promise_batch_create(
            current_id.as_bytes().len() as _,
            current_id.as_bytes().as_ptr() as _,
        );
        // Deploy the contract code.
        sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
        // Call promise to migrate the state.
        // Batched together to fail upgrade if migration fails.
        sys::promise_batch_action_function_call(
            promise_id,
            SELF_MIGRATE_METHOD_NAME.len() as _,
            SELF_MIGRATE_METHOD_NAME.as_ptr() as _,
            0,
            0,
            &NO_DEPOSIT as *const u128 as _,
            (env::prepaid_gas() - env::used_gas() - UPDATE_GAS_LEFTOVER).0,
        );
        sys::promise_return(promise_id);
    }
}

pub(crate) fn upgrade_using_factory(code_hash: Base58CryptoHash) {
    let account_id = get_default_factory_id();
    // Create a promise toward the factory.
    let promise_id = env::promise_batch_create(&account_id);
    // Call `update` method from the factory which calls `update` method on this account.
    env::promise_batch_action_function_call(
        promise_id,
        "update",
        &json!({ "account_id": env::current_account_id(), "code_hash": code_hash })
            .to_string()
            .into_bytes(),
        NO_DEPOSIT,
        env::prepaid_gas() - env::used_gas() - FACTORY_UPDATE_GAS_LEFTOVER,
    );
    env::promise_return(promise_id);
}

#[allow(dead_code)]
/// Self upgrade, optimizes gas by not loading into memory the code.
pub(crate) fn upgrade_self(hash: &[u8]) {
    let current_id = env::current_account_id();
    let method_name = "migrate".as_bytes().to_vec();
    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY;
    unsafe {
        // Load input (wasm code) into register 0.
        sys::storage_read(hash.len() as _, hash.as_ptr() as _, 0);
        // schedule a Promise tx to this same contract
        let promise_id = sys::promise_batch_create(
            current_id.as_bytes().len() as _,
            current_id.as_bytes().as_ptr() as _,
        );
        // 1st item in the Tx: "deploy contract" (code is taken from register 0)
        sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
        // 2nd item in the Tx: call this_contract.migrate() with remaining gas
        sys::promise_batch_action_function_call(
            promise_id,
            method_name.len() as _,
            method_name.as_ptr() as _,
            0 as _,
            0 as _,
            0 as _,
            attached_gas.0,
        );
    }
}

pub(crate) fn upgrade_remote(receiver_id: &AccountId, method_name: &str, hash: &[u8]) {
    unsafe {
        // Load input into register 0.
        sys::storage_read(hash.len() as _, hash.as_ptr() as _, 0);
        let promise_id = sys::promise_batch_create(
            receiver_id.as_bytes().len() as _,
            receiver_id.as_bytes().as_ptr() as _,
        );
        let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_REMOTE_DEPLOY;
        sys::promise_batch_action_function_call(
            promise_id,
            method_name.len() as _,
            method_name.as_ptr() as _,
            u64::MAX as _,
            0 as _,
            0 as _,
            attached_gas.0,
        );
    }
}

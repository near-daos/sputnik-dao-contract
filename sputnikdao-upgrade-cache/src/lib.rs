use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base58CryptoHash};
use near_sdk::{env, sys, near_bindgen, Balance, CryptoHash, PanicOnDefault, Gas};

const CODE_STORAGE_COST: Balance = 6_000_000_000_000_000_000_000_000; // 6 NEAR

/// Leftover gas after creating promise and calling update.
const GAS_UPDATE_LEFTOVER: Gas = Gas(20_000_000_000_000);

// The values used when writing initial data to the storage.
const DAO_CONTRACT_INITIAL_CODE: &[u8] = include_bytes!("../../sputnikdao2/res/sputnikdao2.wasm");

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct SputnikDAOUpgradeCache {}

#[near_bindgen]
impl SputnikDAOUpgradeCache {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    /// Forces update on the calling contract.
    /// Only intended for sputnik v2 DAO's created by sputnik factory
    #[payable]
    pub fn store_contract_self(&mut self) {
        let account_id = env::predecessor_account_id().as_bytes().to_vec();
        let method_name = &"store_blob";

        // TODO: Could lock down contract upgrades to a specific factory, using account like:
        // let dao_id = env::predecessor_account_id().to_string();
        // let idx = dao_id.find('.').expect("INTERNAL_FAIL");
        // // ex: sputnik-dao.near
        // let factory_id = &dao_id[idx + 1..];

        // Confirm payment before proceeding
        assert!(
            CODE_STORAGE_COST <= env::attached_deposit(),
            "Must at least deposit {} to store",
            CODE_STORAGE_COST
        );

        unsafe {
            // Create a promise toward given account.
            let promise_id =
                sys::promise_batch_create(account_id.len() as _, account_id.as_ptr() as _);
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                DAO_CONTRACT_INITIAL_CODE.len() as _,
                DAO_CONTRACT_INITIAL_CODE.as_ptr() as _,
                &env::attached_deposit() as *const u128 as _,
                (env::prepaid_gas() - env::used_gas() - GAS_UPDATE_LEFTOVER).0,
            );
            sys::promise_return(promise_id);
        }
    }

    // TODO: Add FN to remove blob

    /// Return the stored code to check compatibility
    pub fn get_code(&self) -> &[u8] {
        DAO_CONTRACT_INITIAL_CODE
    }

    /// Return the code hash of the stored code to check consistency
    pub fn get_code_hash(&self) -> Base58CryptoHash {
        let code = DAO_CONTRACT_INITIAL_CODE.to_vec();
        let sha256_hash = env::sha256(&code);
        slice_to_hash(&sha256_hash)
    }
}

pub fn slice_to_hash(hash: &[u8]) -> Base58CryptoHash {
    let mut result: CryptoHash = [0; 32];
    result.copy_from_slice(&hash);
    Base58CryptoHash::from(result)
}

// #[cfg(test)]
// mod tests {
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::{testing_env, PromiseResult};

//     use super::*;

//     #[test]
//     fn test_basics() {
//         let mut context = VMContextBuilder::new();
//         testing_env!(context
//             .current_account_id(accounts(0))
//             .predecessor_account_id(accounts(0))
//             .build());
//         let mut factory = SputnikDAOFactory::new();

//         testing_env!(context.attached_deposit(10).build());
//         factory.create("test".parse().unwrap(), "{}".as_bytes().to_vec().into());

//         testing_env!(
//             context.predecessor_account_id(accounts(0)).build(),
//             near_sdk::VMConfig::test(),
//             near_sdk::RuntimeFeesConfig::test(),
//             Default::default(),
//             vec![PromiseResult::Successful(vec![])],
//         );
//         factory.on_create(
//             format!("test.{}", accounts(0)).parse().unwrap(),
//             U128(10),
//             accounts(0),
//         );
//         assert_eq!(
//             factory.get_dao_list(),
//             vec![format!("test.{}", accounts(0)).parse().unwrap()]
//         );
//         assert_eq!(
//             factory.get_daos(0, 100),
//             vec![format!("test.{}", accounts(0)).parse().unwrap()]
//         );
//     }
// }

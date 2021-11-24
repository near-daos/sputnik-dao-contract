use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::PromiseResult;
use near_sdk::{assert_self, env, ext_contract, near_bindgen, AccountId, Gas, Promise, PublicKey};

const CODE: &[u8] = include_bytes!("../../sputnikdao2/res/sputnikdao2.wasm");

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: Gas = Gas(75_000_000_000_000);

/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: Gas = Gas(10_000_000_000_000);

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool;
    fn on_check_daos_activity(&mut self, account_id: AccountId);
}

#[ext_contract(ext_dao)]
pub trait ExtDao {
    fn is_active(&mut self) -> bool;
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct SputnikDAOFactory {
    daos: UnorderedSet<AccountId>,
}

impl Default for SputnikDAOFactory {
    fn default() -> Self {
        env::panic_str("SputnikDAOFactory should be initialized before usage")
    }
}

#[near_bindgen]
impl SputnikDAOFactory {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            daos: UnorderedSet::new(b"d".to_vec()),
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

    // Check daos activity.
    pub fn check_daos_activity(&self) {
        for dao in self.daos.iter() {
            ext_dao::is_active(dao.clone(), 0, Gas(10_000_000_000_000)).then(
                ext_self::on_check_daos_activity(
                    dao.clone(),
                    env::current_account_id(),
                    0,
                    Gas(10_000_000_000_000),
                ),
            );
        }
    }

    pub fn on_check_daos_activity(&mut self, account_id: AccountId) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "DAO Factory Error: This is a callback method"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => panic!(
                "DAO Factory Error: Received PromiseResult::NotReady for {:?}",
                account_id
            ),
            PromiseResult::Failed => panic!(
                "DAO Factory Error: Received PromiseResult::Failed for {:?}",
                account_id
            ),
            PromiseResult::Successful(result) => {
                let is_active = near_sdk::serde_json::from_slice::<bool>(&result).unwrap();
                if !is_active {
                    self.daos.remove(&account_id);
                }
            }
        }
    }

    #[payable]
    pub fn create(
        &mut self,
        name: AccountId,
        public_key: Option<PublicKey>,
        args: Base64VecU8,
    ) -> Promise {
        let account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .parse()
            .unwrap();
        let mut promise = Promise::new(account_id.clone())
            .create_account()
            .deploy_contract(CODE.to_vec())
            .transfer(env::attached_deposit());
        if let Some(key) = public_key {
            promise = promise.add_full_access_key(key.into())
        }
        promise
            .function_call(
                "new".to_string(),
                args.into(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS - ON_CREATE_CALL_GAS,
            )
            .then(ext_self::on_create(
                account_id,
                U128(env::attached_deposit()),
                env::predecessor_account_id(),
                env::current_account_id(),
                0,
                ON_CREATE_CALL_GAS,
            ))
    }

    pub fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool {
        assert_self();
        if near_sdk::is_promise_success() {
            self.daos.insert(&account_id);
            true
        } else {
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, testing_env_with_promise_results, VMContextBuilder};
    use near_sdk::{testing_env, PromiseResult};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.current_account_id(accounts(0)).build());
        let mut factory = SputnikDAOFactory::new();
        testing_env!(context.attached_deposit(10).build());
        factory.create(
            "test".parse().unwrap(),
            Some(
                "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
                    .parse()
                    .unwrap(),
            ),
            "{}".as_bytes().to_vec().into(),
        );
        testing_env_with_promise_results(
            context.predecessor_account_id(accounts(0)).build(),
            PromiseResult::Successful(vec![]),
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

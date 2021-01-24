use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::{Base58PublicKey, Base64VecU8};
use near_sdk::{env, near_bindgen, AccountId, Promise};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

const CODE: &[u8] = include_bytes!("../../sputnikdao/res/sputnikdao.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: u64 = 40_000_000_000_000;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct SputnikDAOFactory {
    daos: UnorderedSet<AccountId>,
}

impl Default for SputnikDAOFactory {
    fn default() -> Self {
        env::panic(b"SputnikDAOFactory should be initialized before usage")
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

    #[payable]
    pub fn create(
        &mut self,
        name: AccountId,
        public_key: Option<Base58PublicKey>,
        args: Base64VecU8,
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id());
        self.daos.insert(&account_id);
        let mut promise = Promise::new(account_id)
            .create_account()
            .deploy_contract(CODE.to_vec())
            .transfer(env::attached_deposit());
        if let Some(key) = public_key {
            promise = promise.add_full_access_key(key.into())
        }
        promise.function_call(
            b"new".to_vec(),
            args.into(),
            0,
            env::prepaid_gas() - CREATE_CALL_GAS,
        )
    }
}

#[cfg(test)]
mod tests {
    use near_std::context::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};

    use super::*;

    #[test]
    fn test_basics() {
        testing_env!(VMContextBuilder::new()
            .current_account_id(accounts(0))
            .finish());
        let mut factory = SputnikDAOFactory::new();
        testing_env!(VMContextBuilder::new()
            .current_account_id(accounts(0))
            .attached_deposit(10)
            .finish());
        factory.create(
            "test".to_string(),
            Some(Base58PublicKey(vec![])),
            "{}".as_bytes().to_vec().into(),
        );
        assert_eq!(
            factory.get_dao_list(),
            vec![format!("test.{}", accounts(0))]
        );
    }
}

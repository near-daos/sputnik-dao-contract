use crate::*;
use near_sdk::{ext_contract, Gas, PromiseOrValue, PromiseResult};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

/// TODO: this should be in the near_standard_contracts
#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait Contract {
    fn exchange_callback_post_withdraw(&mut self, sender_id: AccountId, amount: U128);
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        assert!(msg.is_empty(), "ERR_MSG_INCORRECT");
        assert_eq!(
            token_id,
            self.data()
                .vote_token_id
                .clone()
                .expect("ERR_NO_VOTE_TOKEN")
        );
        self.internal_deposit(sender_id.as_ref(), amount.into());
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl Contract {
    pub fn ft_total_supply(&self) -> U128 {
        assert!(self.data().vote_token_id.is_some(), "ERR_NO_VOTE_TOKEN");
        U128(self.data().vote_token_total_amount)
    }

    pub fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        assert!(self.data().vote_token_id.is_some(), "ERR_NO_VOTE_TOKEN");
        U128(self.internal_get_user(account_id.as_ref()).vote_amount.0)
    }

    pub fn withdraw(&mut self, receiver_id: ValidAccountId, amount: U128) {
        let vote_token_id = self
            .data()
            .vote_token_id
            .clone()
            .expect("ERR_NO_VOTE_TOKEN");
        let sender_id = env::predecessor_account_id();
        self.internal_withdraw(&sender_id, amount.0);
        ext_fungible_token::ft_transfer(
            receiver_id,
            amount,
            None,
            &vote_token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::exchange_callback_post_withdraw(
            sender_id,
            amount,
            &env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER,
        ));
    }

    #[private]
    pub fn exchange_callback_post_withdraw(&mut self, sender_id: AccountId, amount: U128) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_ICALLBACK_POST_WITHDRAW_INVALID",
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                self.internal_deposit(&sender_id, amount.0);
            }
        };
    }
}

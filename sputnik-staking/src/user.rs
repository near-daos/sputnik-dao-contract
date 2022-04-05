use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, Duration, StorageUsage};

use crate::*;

const U64_LEN: StorageUsage = 8;
const U128_LEN: StorageUsage = 16;
const ACCOUNT_MAX_LENGTH: StorageUsage = 64;

/// User data.
/// Recording deposited voting tokens, storage used and delegations for voting.
/// Once delegated - the tokens are used in the votes. It records for each delegate when was the last vote.
/// When undelegating - the new delegations or withdrawal are only available after cooldown period from last vote of the delegate.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Total amount of storage used by this user struct.
    pub storage_used: StorageUsage,
    /// Amount of $NEAR to cover storage.
    pub near_amount: U128,
    /// Amount of staked token deposited.
    pub vote_amount: U128,
    /// Withdrawal or next delegation available timestamp.
    pub next_action_timestamp: U64,
    /// List of delegations to other accounts.
    pub delegated_amounts: Vec<(AccountId, U128)>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedUser {
    Default(User),
}

impl User {
    pub fn new(near_amount: Balance) -> Self {
        Self {
            storage_used: Self::min_storage(),
            near_amount: U128(near_amount),
            vote_amount: U128(0),
            delegated_amounts: vec![],
            next_action_timestamp: 0.into(),
        }
    }

    /// Minimum storage with empty delegations in bytes.
    /// This includes u128 stored in DAO for delegations to this user.
    /// They are deposited on internal_register and removed on internal_unregister.
    pub fn min_storage() -> StorageUsage {
        ACCOUNT_MAX_LENGTH + 2 * U64_LEN + 4 * U128_LEN
    }

    fn assert_storage(&self) {
        assert!(
            (self.storage_used as Balance) * env::storage_byte_cost() <= self.near_amount.0,
            "ERR_NOT_ENOUGH_STORAGE"
        );
    }

    pub(crate) fn delegated_amount(&self) -> Balance {
        self.delegated_amounts
            .iter()
            .fold(0, |total, (_, amount)| total + amount.0)
    }

    /// Record delegation from this account to another account.
    /// Fails if not enough available balance to delegate.
    pub fn delegate(&mut self, delegate_id: AccountId, amount: Balance) {
        assert!(
            self.delegated_amount() + amount <= self.vote_amount.0,
            "ERR_NOT_ENOUGH_AMOUNT"
        );
        assert!(
            env::block_timestamp() >= self.next_action_timestamp.0,
            "ERR_NOT_ENOUGH_TIME_PASSED"
        );
        self.storage_used += delegate_id.as_bytes().len() as StorageUsage + U128_LEN;
        self.delegated_amounts.push((delegate_id, U128(amount)));
        self.assert_storage();
    }

    /// Remove given amount from delegates. Updates timestamp when next action can be called.
    /// Fails if delegate not found or not enough amount delegated.
    pub fn undelegate(
        &mut self,
        delegate_id: &AccountId,
        amount: Balance,
        undelegation_period: Duration,
    ) {
        let f = self
            .delegated_amounts
            .iter()
            .enumerate()
            .find(|(_, (account_id, _))| account_id == delegate_id)
            .expect("ERR_NO_DELEGATE");
        let element = (f.0, ((f.1).1).0);
        assert!(element.1 >= amount, "ERR_NOT_ENOUGH_AMOUNT");
        if element.1 == amount {
            self.delegated_amounts.remove(element.0);
            self.storage_used -= delegate_id.as_bytes().len() as StorageUsage + U128_LEN;
        } else {
            (self.delegated_amounts[element.0].1).0 -= amount;
        }
        self.next_action_timestamp = (env::block_timestamp() + undelegation_period).into();
    }

    /// Withdraw the amount.
    /// Fails if there is not enough available balance.
    pub fn withdraw(&mut self, amount: Balance) {
        assert!(
            self.delegated_amount() + amount <= self.vote_amount.0,
            "ERR_NOT_ENOUGH_AVAILABLE_AMOUNT"
        );
        assert!(
            env::block_timestamp() >= self.next_action_timestamp.0,
            "ERR_NOT_ENOUGH_TIME_PASSED"
        );
        self.vote_amount.0 -= amount;
    }

    /// Deposit given amount of vote tokens.
    pub fn deposit(&mut self, amount: Balance) {
        self.vote_amount.0 += amount;
    }

    /// Returns amount in NEAR that is available for storage.
    pub fn storage_available(&self) -> Balance {
        self.near_amount.0 - self.storage_used as Balance * env::storage_byte_cost()
    }
}

impl Contract {
    pub fn internal_get_user(&self, account_id: &AccountId) -> User {
        self.internal_get_user_opt(account_id).expect("NO_USER")
    }

    pub fn internal_get_user_opt(&self, account_id: &AccountId) -> Option<User> {
        self.users
            .get(account_id)
            .map(|versioned_user| match versioned_user {
                VersionedUser::Default(user) => user,
            })
    }

    pub fn save_user(&mut self, account_id: &AccountId, user: User) {
        self.users.insert(account_id, &VersionedUser::Default(user));
    }

    /// Internal register new user.
    pub fn internal_register_user(&mut self, sender_id: &AccountId, near_amount: Balance) {
        let user = User::new(near_amount);
        self.save_user(sender_id, user);
        ext_sputnik::register_delegation(
            sender_id.clone(),
            self.owner_id.clone(),
            (U128_LEN as Balance) * env::storage_byte_cost(),
            GAS_FOR_REGISTER,
        );
    }

    /// Deposit voting token.
    pub fn internal_deposit(&mut self, sender_id: &AccountId, amount: Balance) {
        let mut sender = self.internal_get_user(&sender_id);
        sender.deposit(amount);
        self.save_user(&sender_id, sender);
        self.total_amount += amount;
    }

    /// Withdraw voting token.
    pub fn internal_withdraw(&mut self, sender_id: &AccountId, amount: Balance) {
        let mut sender = self.internal_get_user(&sender_id);
        sender.withdraw(amount);
        self.save_user(&sender_id, sender);
        assert!(self.total_amount >= amount, "ERR_INTERNAL");
        self.total_amount -= amount;
    }

    /// Given user delegates given amount of votes to another user.
    /// The other user must be registered.
    pub fn internal_delegate(
        &mut self,
        sender_id: AccountId,
        delegate_id: AccountId,
        amount: Balance,
    ) {
        let mut sender = self.internal_get_user(&sender_id);
        assert!(self.users.contains_key(&delegate_id), "ERR_NOT_REGISTERED");
        sender.delegate(delegate_id.clone(), amount);
        self.save_user(&sender_id, sender);
    }

    /// Undelegate votes from given delegate.
    pub fn internal_undelegate(
        &mut self,
        sender_id: AccountId,
        delegate_id: AccountId,
        amount: Balance,
    ) {
        let mut sender = self.internal_get_user(&sender_id);
        sender.undelegate(&delegate_id, amount, self.unstake_period);
        self.save_user(&sender_id, sender);
    }
}

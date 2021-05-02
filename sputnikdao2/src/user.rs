//! User information of the given DAO.

use near_sdk::{AccountId, Balance, StorageUsage};

use crate::*;

const U64_LEN: StorageUsage = 8;
const U128_LEN: StorageUsage = 16;
const ACCOUNT_MAX_LENGTH: StorageUsage = 64;

/// User data.
/// Recording deposited voting tokens, storage used and delegations for voting.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Storage used.
    pub storage_used: StorageUsage,
    /// Amount of NEAR deposited for storage.
    pub near_amount: U128,
    /// Amount of vote token deposited.
    pub vote_amount: U128,
    /// Accumulated delegated weight.
    pub vote_weight: U128,
    /// Delegations.
    pub delegated_weight: Vec<(AccountId, U128)>,
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
            delegated_weight: Vec::default(),
            vote_weight: U128(0),
        }
    }

    pub fn min_storage() -> StorageUsage {
        ACCOUNT_MAX_LENGTH + U64_LEN + 3 * U128_LEN
    }

    fn assert_storage(&self) {
        assert!(
            (self.storage_used as Balance) * env::storage_byte_cost() <= self.near_amount.0,
            "ERR_NOT_ENOUGH_STORAGE"
        );
    }

    fn delegated_amount(&self) -> Balance {
        self.delegated_weight
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
        self.storage_used += delegate_id.len() as StorageUsage + U128_LEN;
        self.delegated_weight.push((delegate_id, U128(amount)));
        self.assert_storage();
    }

    /// Remove given amount from delegates.
    /// Fails if delegate not found or not enough amount delegated.
    pub fn undelegate(&mut self, delegate_id: &AccountId, amount: Balance) {
        let f = self
            .delegated_weight
            .iter()
            .enumerate()
            .find(|(_, (account_id, _))| account_id == delegate_id)
            .expect("ERR_NO_DELEGATE");
        let element = (f.0, ((f.1).1).0);
        assert!(element.1 <= amount, "ERR_NOT_ENOUGH_AMOUNT");
        if element.1 == amount {
            self.delegated_weight.remove(element.0);
            self.storage_used -= delegate_id.len() as StorageUsage + U128_LEN;
        } else {
            (self.delegated_weight[element.0].1).0 -= amount;
        }
    }

    /// Record when someone delegates to this account.
    pub fn delegate_to(&mut self, amount: Balance) {
        self.vote_weight.0 += amount;
    }

    /// Record when someone undelegates from this account.
    pub fn delegate_from(&mut self, amount: Balance) {
        self.vote_weight.0 -= amount;
    }

    /// Withdraw the amount. Fails if there is more delegated.
    pub fn withdraw(&mut self, amount: Balance) {
        assert!(
            self.delegated_amount() + amount <= self.vote_amount.0,
            "ERR_NOT_ENOUGH_AVAILABLE_AMOUNT"
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
        self.data()
            .users
            .get(account_id)
            .map(|versioned_user| match versioned_user {
                VersionedUser::Default(user) => user,
            })
    }

    pub fn save_user(&mut self, account_id: &AccountId, user: User) {
        self.data_mut()
            .users
            .insert(account_id, &VersionedUser::Default(user));
    }

    pub fn get_user_weight(&self, account_id: &AccountId) -> Balance {
        self.internal_get_user_opt(account_id)
            .map(|user| user.vote_weight.0)
            .unwrap_or_default()
    }

    /// Internal register new user.
    pub fn internal_register_user(&mut self, sender_id: &AccountId, near_amount: Balance) {
        let user = User::new(near_amount);
        self.save_user(sender_id, user);
    }

    /// Deposit voting token.
    pub fn internal_deposit(&mut self, sender_id: &AccountId, amount: Balance) {
        let mut sender = self.internal_get_user(&sender_id);
        sender.deposit(amount);
        self.save_user(&sender_id, sender);
        self.data_mut().vote_token_total_amount += amount;
    }

    /// Withdraw voting token.
    pub fn internal_withdraw(&mut self, sender_id: &AccountId, amount: Balance) {
        let mut sender = self.internal_get_user(&sender_id);
        sender.withdraw(amount);
        self.save_user(&sender_id, sender);
        self.data_mut().vote_token_total_amount -= amount;
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
        sender.delegate(delegate_id.clone(), amount);
        if sender_id != delegate_id {
            let mut delegate = self.internal_get_user(&delegate_id);
            delegate.delegate_to(amount);
            self.save_user(&delegate_id, delegate);
        } else {
            sender.delegate_to(amount);
        }
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
        sender.undelegate(&delegate_id, amount);
        if delegate_id != sender_id {
            let mut delegate = self.internal_get_user(&delegate_id);
            delegate.delegate_from(amount);
            self.save_user(&delegate_id, delegate);
        } else {
            sender.delegate_from(amount);
        }
        self.save_user(&sender_id, sender);
    }
}

#[near_bindgen]
impl Contract {
    /// Delegate given amount to the delegate account.
    pub fn delegate_vote(&mut self, delegate_id: ValidAccountId, amount: U128) {
        self.internal_delegate(
            env::predecessor_account_id(),
            delegate_id.as_ref().clone(),
            amount.0,
        );
    }

    /// Remove given amount of delegation.
    pub fn undelegate_vote(&mut self, delegate_id: ValidAccountId, amount: U128) {
        self.internal_undelegate(
            env::predecessor_account_id(),
            delegate_id.as_ref().clone(),
            amount.0,
        );
    }
}

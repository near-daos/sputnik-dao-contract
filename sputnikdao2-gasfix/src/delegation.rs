use crate::*;

impl Contract {
    pub fn get_user_weight(&self, account_id: &AccountId) -> Balance {
        self.delegations.get(account_id).unwrap_or_default()
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register_delegation(&mut self, account_id: &AccountId) {
        let staking_id = self.staking_id.clone().expect("ERR_NO_STAKING");
        assert_eq!(
            env::predecessor_account_id(),
            staking_id,
            "ERR_INVALID_CALLER"
        );
        assert_eq!(env::attached_deposit(), 16 * env::storage_byte_cost());
        self.delegations.insert(account_id, &0);
    }

    pub fn delegate(&mut self, account_id: &AccountId, amount: U128) {
        let staking_id = self.staking_id.clone().expect("ERR_NO_STAKING");
        assert_eq!(
            env::predecessor_account_id(),
            staking_id,
            "ERR_INVALID_CALLER"
        );
        let prev_amount = self
            .delegations
            .get(account_id)
            .expect("ERR_NOT_REGISTERED");
        self.delegations
            .insert(account_id, &(prev_amount + amount.0));
        self.total_delegation_amount += amount.0;
    }

    pub fn undelegate(&mut self, account_id: &AccountId, amount: U128) {
        let staking_id = self.staking_id.clone().expect("ERR_NO_STAKING");
        assert_eq!(
            env::predecessor_account_id(),
            staking_id,
            "ERR_INVALID_CALLER"
        );
        let prev_amount = self.delegations.get(account_id).unwrap_or_default();
        assert!(prev_amount >= amount.0, "ERR_INVALID_STAKING_CONTRACT");
        self.delegations
            .insert(account_id, &(prev_amount - amount.0));
        self.total_delegation_amount -= amount.0;
    }
}

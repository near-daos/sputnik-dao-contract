use crate::*;
use near_sdk::{env, near_bindgen, AccountId};

#[near_bindgen]
impl Contract {
    /// Removes a user from all roles.  
    /// Returns `true` if the user was removed from at least one role.
    ///
    /// The required parameters are used for checking against the caller
    /// and the DAO's name to help in preventing mistakes.
    pub fn quit_from_all_roles(&mut self, user: AccountId, dao: String) -> bool {
        let quitting_member = env::predecessor_account_id();
        if quitting_member != user {
            env::panic_str("ERR_QUIT_WRONG_ACC");
        }

        let dao_name = self.get_config().name;
        if dao_name != dao {
            env::panic_str("ERR_QUIT_WRONG_DAO");
        }

        let mut new_policy = self.policy.get().unwrap().to_policy();
        let removed = new_policy.remove_member_from_all_roles(&quitting_member);
        self.policy
            .set(&crate::VersionedPolicy::Current(new_policy));
        removed
    }
}

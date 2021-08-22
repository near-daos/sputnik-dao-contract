use near_sdk::AccountId;
use serde::*;
use std::fmt::Display;

// TODO - create const-friendly AccountId2

#[derive(Serialize,Deserialize)]
pub struct NftApprove {
    pub token_id:String,
    // #[serde(borrow)]
    pub account_id:AccountId,
    pub msg:Option<String>
}


pub struct CallBuild<'a> {
    pub arg:String,
    pub amount:Option<f64>,
    pub account_id:AccountId,
    pub contract:&'a str,
    pub gas:Option<usize>
}

pub struct DeployBuild<'a> {
    pub wasm_file:&'a str,
    pub init_fn:Option<&'a str>,
    pub init_args:Option<&'a str>,
    pub account_id:AccountId,
}

pub struct CreateAccountBuild<'a> {
    pub account_id:&'a AccountId,
    pub master_account:&'a AccountId,
    pub initial_balance:u16,
}


use crate::*;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use std::ops::Add;
use std::fmt::Display;
use std::process::Command;

use near_sdk::AccountId;
use std::str::FromStr;


impl<T> ToJson for T where T:Serialize+DeserializeOwned {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

// impl<T> ToJson for T where T:Serialize+DeserializeOwned+for<'a> Deserialize<'a> {
//     fn to_json(&self) -> String {
//         serde_json::to_string(&self).unwrap()
//     }
// }

impl NearValue for u128 {
    fn to_near_u128(self) -> String {
        (self * 10u128.pow(24)).to_string()
    }
    fn to_ynear_u128(self) -> String {
        (self / 10u128.pow(24)).to_string()
    }
}

impl NearValue for f64 {
    fn to_near_u128(self) -> String {
        (self * 10f64.powf(24.0)).to_string()
    }
    fn to_ynear_u128(self) -> String {
        (&(self / 10f64.powf(24.0)).to_string()[0..26]).to_string()
    }
}

impl YoctoString for f64 {
    fn yocto_string(self) -> String {
        (&(self / 10f64.powf(24.0)).to_string()[0..26]).to_string()
    }
}

impl Yocto for f64 {
    fn yocto(self) -> Self {
        self / 10f64.powf(24.0)
    }

    fn near(self) -> Self {
        self * 10f64.powf(24.0)
    }
}

impl Yocto for u128 {
    fn yocto(self) -> Self {
        self / 10u128.pow(24)
    }

    fn near(self) -> Self {
        self * 10u128.pow(24)
    }
}

impl ContractMethod for NftApprove {
    fn contract_method(&self) -> String {
        "nft_approve".into()
    }
}

impl<T> CallBuilder for T where T:ContractMethod+ToJson {
    fn call_builder(&self) -> CallBuild {
        CallBuild {
            arg:format!("{} '{}'",self.contract_method(), self.to_json()),
            amount: None,
            contract:&EMPTY_STRING,
            account_id:AccountId::new_unchecked("a.near".into()),
            gas: None
        }
    }
}

impl<T> DeployBuilder for T where T:AsRef<str> {
    fn deploy_builder(&self) -> DeployBuild {
        DeployBuild{
            wasm_file: self.as_ref(),
            init_fn: None,
            init_args: None,
            account_id: AccountId::from_str("a.near").unwrap(),
        }
    }
}

impl CreateAccountBuilder for AccountId {
    fn create_account_builder(&self) -> CreateAccountBuild {
        CreateAccountBuild{ account_id: self, master_account: self, initial_balance: 0 }
    }
}

impl<'a> CreateAccountBuild<'a> {
    pub fn master_account(mut self, a:&'a AccountId) -> CreateAccountBuild<'a> {
        self.master_account = a;
        self
    }
    pub fn initial_balance(mut self, i:u16)-> CreateAccountBuild<'a> {
        self.initial_balance = i;
        self
    }
    pub fn build(self)->String{
        let mut s = format!("{} --masterAccount {} --initialBalance {}",self.account_id,self.master_account,self.initial_balance);
        s
    }
}

impl<'a> DeployBuild<'a> {
    pub fn account_id(mut self, a:AccountId)-> DeployBuild<'a> {
        self.account_id = a;
        self
    }
    pub fn wasm_file(mut self, s:&'a str)-> DeployBuild<'a> {
        self.wasm_file = s;
        self
    }
    pub fn init_fn(mut self, s:Option<&'a str>)-> DeployBuild<'a> {
        self.init_fn = s;
        self
    }
    pub fn init_args(mut self, s:Option<&'a str>)-> DeployBuild<'a> {
        self.init_args = s;
        self
    }
    pub fn build(self)->String{
        let mut s = format!("--wasmFile {} --accountId {}",self.wasm_file,self.account_id);
        if let Some(v) = self.init_fn {
            s = format!("{} --initFunction '{}'",s,v);
        }
        if let Some(v) = self.init_args {
            s = format!("{} --initArgs '{}'",s,v);
        }
        s
    }
}


impl<'a> CallBuild<'a> {
    pub fn amount(mut self, near_amount:Option<f64>)-> CallBuild<'a> {
        self.amount = near_amount;
        self
    }
    pub fn account_id(mut self, a:AccountId)-> CallBuild<'a> {
        self.account_id = a;
        self
    }
    pub fn gas(mut self, g:Option<usize>)-> CallBuild<'a> {
        self.gas = g;
        self
    }
    pub fn contract(mut self, s:&'a str)-> CallBuild<'a> {
        self.contract = s;
        self
    }
    pub fn build(self)->String{
        let mut s = format!("{} {} --accountId {}",self.contract,self.arg,self.account_id);
        if let Some(v) = self.amount {
            s = format!("{} --amount {}",s,v);
        }
        if let Some(v) = self.gas {
            s = format!("{} --gas {}",s,v);
        }
        s
    }
}

impl Print for String {
    fn print(&self) {
        println!("{}",self)
    }
}

// TODO - change this to enum
impl NearCommand for String {
    fn near_call(&self) {
        near_command("call",self)
    }

    fn near_view(&self) {
        near_command("view",self)
    }

    fn near_deploy(&self) {
        near_command("deploy",self)
    }

    fn near_create_account(&self) {
        near_command("create-account",self)

    }
}

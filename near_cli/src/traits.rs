use crate::{CallBuild, DeployBuild, CreateAccountBuild};

pub trait ToJson {
    fn to_json(&self)->String;
}

pub trait NearValue {
    fn to_near_u128(self)->String;
    fn to_ynear_u128(self) ->String;
}

pub trait Yocto {
    fn yocto(self)->Self;
    fn near(self)->Self;
}

pub trait YoctoString {
    fn yocto_string(self)->String;
}

pub trait ContractMethod {
    fn contract_method(&self) ->String;
}

pub trait NearCommand {
    fn near_call(&self);
    fn near_view(&self);
    fn near_deploy(&self);
    fn near_create_account(&self);
}

pub trait Print {
    fn print(&self);
}

pub trait CallBuilder {
    fn call_builder(&self)-> CallBuild;

}

pub trait DeployBuilder {
    fn deploy_builder(&self)-> DeployBuild;
}

pub trait CreateAccountBuilder {
    fn create_account_builder(&self)-> CreateAccountBuild;
}

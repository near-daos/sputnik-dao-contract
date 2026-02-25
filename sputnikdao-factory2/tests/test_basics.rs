use near_api::{AccountId, NearToken, RPCEndpoint};
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use near_sdk::serde_json::{Value, json};
use near_sdk::{
    AccountIdRef,
    base64::{Engine as _, engine::general_purpose},
};
use std::sync::OnceLock;

static FACTORY_WASM: OnceLock<Vec<u8>> = OnceLock::new();
static DAO_WASM: OnceLock<Vec<u8>> = OnceLock::new();

fn factory_wasm_bytes() -> &'static [u8] {
    FACTORY_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(Default::default())
            .expect("Failed to build sputnikdao-factory2");
        std::fs::read(&wasm_path).expect("Failed to read sputnikdao_factory2.wasm")
    })
}

fn dao_wasm_bytes() -> &'static [u8] {
    DAO_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
            manifest_path: Some(
                concat!(env!("CARGO_MANIFEST_DIR"), "/../sputnikdao2/Cargo.toml").into(),
            ),
            ..Default::default()
        })
        .expect("Failed to build sputnikdao2");
        std::fs::read(&wasm_path).expect("Failed to read sputnikdao2.wasm")
    })
}

#[tokio::test]
async fn test_factory() -> testresult::TestResult {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
        AccountIdRef::new_or_panic("sputnik-dao.near");
    let sputnikdao_factory_contract =
        near_api::Contract(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned());

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = near_api::Signer::from_secret_key(DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?)?;

    sandbox
        .import_account(
            RPCEndpoint::mainnet().url,
            sputnikdao_factory_contract.0.clone(),
        )
        .initial_balance(NearToken::from_near(100))
        .send()
        .await?;

    near_api::Contract::deploy(sputnikdao_factory_contract.0.clone())
        .use_code(factory_wasm_bytes().to_vec())
        .with_init_call("new", ())?
        .max_gas()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .into_result()?;

    // The factory starts with no default code hash, so we store the DAO wasm and register it.
    let dao_hash: near_sdk::json_types::Base58CryptoHash = sputnikdao_factory_contract
        .call_function_raw("store", dao_wasm_bytes().to_vec())
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(50))
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;
    sputnikdao_factory_contract
        .call_function("set_default_code_hash", json!({ "code_hash": dao_hash }))
        .transaction()
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .into_result()?;

    let dao_name = "testdao";
    let create_dao_args = json!({
        "config": {
            "name": dao_name,
            "purpose": "creating dao treasury",
            "metadata": "",
        },
        "policy": {
            "roles": [
                {
                    "kind": {
                        "Group": ["acc3.near", "acc2.near", "acc1.near"],
                    },
                    "name": "Create Requests",
                    "permissions": [
                        "call:AddProposal",
                        "transfer:AddProposal",
                        "config:Finalize",
                    ],
                    "vote_policy": {},
                },
                {
                    "kind": {
                        "Group": ["acc1.near"],
                    },
                    "name": "Manage Members",
                    "permissions": [
                        "config:*",
                        "policy:*",
                        "add_member_to_role:*",
                        "remove_member_from_role:*",
                    ],
                    "vote_policy": {},
                },
                {
                    "kind": {
                        "Group": ["acc1.near", "acc2.near"],
                    },
                    "name": "Vote",
                    "permissions": ["*:VoteReject", "*:VoteApprove", "*:VoteRemove"],
                    "vote_policy": {},
                },
            ],
            "default_vote_policy": {
                "weight_kind": "RoleWeight",
                "quorum": "0",
                "threshold": [1, 2],
            },
            "proposal_bond": "100000000000000000000000",
            "proposal_period": "604800000000000",
            "bounty_bond": "100000000000000000000000",
            "bounty_forgiveness_period": "604800000000000",
        },
    });

    let alice_account_id: AccountId = DEFAULT_GENESIS_ACCOUNT.sub_account("alice")?;
    sandbox
        .create_account(alice_account_id.clone())
        .initial_balance(NearToken::from_near(100))
        .send()
        .await?;

    sputnikdao_factory_contract
        .call_function(
            "create",
            json!({
                "name": dao_name,
                "args":  general_purpose::STANDARD.encode(create_dao_args.to_string())
            }),
        )
        .transaction()
        .max_gas()
        .deposit(NearToken::from_millinear(10))
        .with_signer(alice_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .into_result()?;

    let dao_account_id: AccountId = SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.sub_account(dao_name)?;

    let get_config_result: Value = near_api::Contract(dao_account_id)
        .call_function("get_config", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    assert_eq!(create_dao_args["config"], get_config_result);

    Ok(())
}

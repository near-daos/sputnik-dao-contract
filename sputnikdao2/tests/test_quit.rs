use crate::utils::{add_member_to_role_proposal, add_proposal, setup_dao};
use near_sdk_sim::{call, to_yocto, view};
use sputnikdao2::{Action, ProposalInput, ProposalKind, VersionedPolicy};
use std::collections::HashMap;

mod utils;

fn user(id: u32) -> String {
    format!("user{}", id)
}

/// Issue #41 "Quitting the DAO" tests
///
///
#[test]
fn test_quitting_the_dao() {
    use near_sdk_sim::UserAccount;
    use sputnikdao2::{Policy, RoleKind, RolePermission};
    use std::collections::HashSet;

    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let user4 = root.create_user(user(4), to_yocto("1000"));

    let new_role = |name: String| RolePermission {
        name,
        kind: RoleKind::Group(HashSet::new()),
        permissions: HashSet::new(),
        vote_policy: HashMap::new(),
    };
    let role_none = new_role("has_nobody".to_string());
    let role_2 = new_role("has_2".to_string());
    let role_3 = new_role("has_3".to_string());
    let role_23 = new_role("has_23".to_string());
    let role_234 = new_role("has_234".to_string());

    let mut policy = view!(dao.get_policy()).unwrap_json::<Policy>();
    policy
        .roles
        .extend(vec![role_none, role_2, role_3, role_23, role_234]);
    add_proposal(
        &root,
        &dao,
        ProposalInput {
            description: "new_policy".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(policy.clone()),
            },
        },
    )
    .assert_success();
    let change_policy = view!(dao.get_last_proposal_id()).unwrap_json::<u64>();
    assert_eq!(change_policy, 1);
    call!(
        root,
        dao.act_proposal(change_policy - 1, Action::VoteApprove, None)
    )
    .assert_success();

    let add_to_roles = |user: &UserAccount, roles: Vec<&str>| {
        for role in roles {
            add_member_to_role_proposal(&root, &dao, user.account_id.clone(), role.to_string())
                .assert_success();

            // approval
            let proposal = view!(dao.get_last_proposal_id()).unwrap_json::<u64>();
            call!(
                root,
                dao.act_proposal(proposal - 1, Action::VoteApprove, None)
            )
            .assert_success();
        }
    };
    add_to_roles(&user2, vec!["has_2", "has_23", "has_234"]);
    add_to_roles(&user3, vec!["has_3", "has_23", "has_234"]);
    add_to_roles(&user4, vec!["has_234"]);

    let role_members = |role_permission: &sputnikdao2::RolePermission| -> Vec<String> {
        if let RoleKind::Group(ref members) = role_permission.kind {
            let mut members = members.into_iter().cloned().collect::<Vec<_>>();
            members.sort();
            members
        } else {
            vec![]
        }
    };

    // quits and returns the remaining role names and their members
    // this is a Vec so the order is preserved
    type RoleNamesAndMembers = Vec<(String, Vec<String>)>;
    type RoleNamesAndMembersRef<'a> = Vec<(&'a str, Vec<&'a str>)>;

    // return role names and members form a dao
    let dao_roles = || -> RoleNamesAndMembers {
        view!(dao.get_policy())
            .unwrap_json::<Policy>()
            .roles
            .into_iter()
            .map(|role_permission| (role_permission.name.clone(), role_members(&role_permission)))
            .collect()
    };

    // makes references into a RoleNamesAndMembers
    // so they are easier to compare against
    fn dao_roles_ref(dao_roles: &RoleNamesAndMembers) -> RoleNamesAndMembersRef {
        dao_roles
            .iter()
            .map(|(name, members)| {
                (
                    name.as_str(),
                    members.into_iter().map(|s| s.as_str()).collect(),
                )
            })
            .collect::<Vec<(&str, Vec<&str>)>>()
    }

    let quit =
        |user: &UserAccount, user_check: &UserAccount, dao_name: String| -> Result<bool, String> {
            use near_sdk_sim::transaction::ExecutionStatus;
            use near_sdk_sim::ExecutionResult;
            let res: ExecutionResult = call!(
                user,
                dao.quit_from_all_roles(user_check.account_id.clone(), dao_name),
                deposit = to_yocto("0")
            );
            match res.status() {
                ExecutionStatus::SuccessValue(_bytes) => Ok(res.unwrap_json::<bool>()),
                ExecutionStatus::Failure(err) => Err(err.to_string()),
                _ => panic!("unexpected status"),
            }
        };

    // initial check,
    // when nobody has quit yet
    let roles = dao_roles();
    {
        assert_eq!(
            dao_roles_ref(&roles),
            vec![
                ("all", vec![]),
                ("council", vec!["root"]),
                ("has_nobody", vec![]),
                ("has_2", vec!["user2",]),
                ("has_3", vec!["user3"]),
                ("has_23", vec!["user2", "user3"]),
                ("has_234", vec!["user2", "user3", "user4"])
            ]
        );
    }

    let config = view!(dao.get_config()).unwrap_json::<sputnikdao2::Config>();
    let dao_name = &config.name;

    // user2 quits
    let res = quit(&user2, &user2, dao_name.clone()).unwrap();
    assert!(res);
    let roles = dao_roles();
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec!["root"]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec!["user3"]),
            ("has_23", vec!["user3"]),
            ("has_234", vec!["user3", "user4"])
        ]
    );

    // user2 quits again
    // makes no change
    let res = quit(&user2, &user2, dao_name.clone()).unwrap();
    assert!(!res);
    let roles = dao_roles();
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec!["root"]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec!["user3"]),
            ("has_23", vec!["user3"]),
            ("has_234", vec!["user3", "user4"])
        ]
    );

    // user3 quits incorrectly, passing the wrong user name
    let res = quit(&user3, &user2, dao_name.clone()).unwrap_err();
    assert_eq!(
        res,
        "Action #0: Smart contract panicked: ERR_QUIT_WRONG_ACC"
    );
    let roles = dao_roles();
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec!["root"]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec!["user3"]),
            ("has_23", vec!["user3"]),
            ("has_234", vec!["user3", "user4"])
        ]
    );

    // user3 quits incorrectly, passing the wrong dao name
    let wrong_dao_name = format!("wrong_{}", &dao_name);
    let res = quit(&user3, &user3, wrong_dao_name).unwrap_err();
    assert_eq!(
        res,
        "Action #0: Smart contract panicked: ERR_QUIT_WRONG_DAO"
    );
    let roles = dao_roles();
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec!["root"]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec!["user3"]),
            ("has_23", vec!["user3"]),
            ("has_234", vec!["user3", "user4"])
        ]
    );

    // user3 quits
    let res = quit(&user3, &user3, dao_name.clone()).unwrap();
    assert!(res);
    let roles = dao_roles();
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec!["root"]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![]),
            ("has_23", vec![]),
            ("has_234", vec!["user4"])
        ]
    );
}

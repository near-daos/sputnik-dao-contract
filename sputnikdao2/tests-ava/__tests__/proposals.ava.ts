import { toYocto, NearAccount, captureError, BN } from 'near-workspaces-ava';

import { workspace, initTestToken, initStaking, setStakingId } from './utils';

async function voteApprove(root: NearAccount, dao: NearAccount, proposalId: number) {
    await root.call(dao, 'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove'
        })
}

workspace.test('basic', async (test, { alice, root, dao }) => {
    test.true(await alice.exists())
    test.true(await root.exists())
    test.true(await dao.exists())
    test.log(await dao.view('get_config'))
})

/*
reference

pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
}

pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

pub enum ProposalKind {
    /// Change the DAO config.
    ChangeConfig { config: Config },
    /// Change the full policy.
    ChangePolicy { policy: VersionedPolicy },
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    FunctionCall {
        receiver_id: AccountId,
        actions: Vec<ActionCall>,
    },
    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base58CryptoHash },
    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base58CryptoHash,
    },
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    /// If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token.
    /// For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
    Transfer {
        /// Can be "" for $NEAR or a valid account id.
        #[serde(with = "serde_with::rust::string_empty_as_none")]
        token_id: Option<AccountId>,
        receiver_id: AccountId,
        amount: U128,
        msg: Option<String>,
    },
    /// Sets staking contract. Can only be proposed if staking contract is not set yet.
    SetStakingContract { staking_id: AccountId },
    /// Add new bounty.
    AddBounty { bounty: Bounty },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
}
*/


workspace.test('add proposal with 1 near', async (test, { alice, root, dao }) => {
    test.is(await dao.view('get_last_proposal_id'), 0);
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    },
        { attachedDeposit: toYocto('1') })
    test.is(await dao.view('get_last_proposal_id'), 1);

    let new_proposal: any = await dao.view('get_proposal', {id: 0})

    test.log(new_proposal);
    test.is(new_proposal.description, 'rename the dao');
    test.is(new_proposal.proposer, 'alice.test.near')
    test.is(new_proposal.status, 'InProgress')

    test.truthy(new_proposal.kind.ChangeConfig)
    test.is(new_proposal.kind.ChangeConfig.config.name, 'sputnikdao')
    //same config as we did not execute that proposal
    test.deepEqual(await dao.view('get_config'), { name: 'sputnik', purpose: 'testing', metadata: '' })
})

workspace.test('add proposal with 0.999... near', async (test, { alice, root, dao }) => {
    test.is(await dao.view('get_last_proposal_id'), 0);
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    let err = await captureError(async () =>
        await alice.call_raw(dao, 'add_proposal', {
            proposal: {
                description: 'rename the dao',
                kind: {
                    ChangeConfig: {
                        config
                    }
                }
            },
        },
            { attachedDeposit: new BN(toYocto('1')).subn(1) })
    )

    test.log(err.toString());
    test.true(err.includes('ERR_MIN_BOND'));
    //the proposal did not count
    test.is(await dao.view('get_last_proposal_id'), 0);

})

workspace.test('voting not allowed for non councils', async (test, {alice, root, dao})=>{
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    //add_proposal returns new proposal id
    const id = await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    }, {attachedDeposit: toYocto('1')})

    //here alice tries to vote for her proposal but she is not a council and has no permission to vote.
    const err = await captureError(async () => await alice.call(dao, 'act_proposal', {
        id,
        action: 'VoteApprove',
        memo: 'trying to vote without permission'
    }))

    test.log(err)
    test.true(err.includes('ERR_PERMISSION_DENIED'))

    let proposal: any = await dao.view('get_proposal', {id});

    test.log(proposal);
    test.is(proposal.status, 'InProgress')
})


workspace.test('voting is allowed for councils', async (test, {alice, root, dao})=>{
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    //alice adds a new proposal
    const id = await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    }, {attachedDeposit: toYocto('1')})

    //council (root) votes on alice's promise
    const res = await root.call(dao, 'act_proposal', {
        id,
        action: 'VoteApprove',
        memo: 'as a council I can vote'
    })

    test.log(res)

    let proposal: any = await dao.view('get_proposal', {id});

    test.log(proposal);
    test.is(proposal.status, 'Approved')

    // proposal approved so now the config is equal to what alice did proposed
    test.deepEqual(await dao.view('get_config'), config) 
})

workspace.test('Proposal ChangePolicy', async (test, {alice, root, dao})=>{
    let policy = [root.accountId];
    let errorString = await captureError(async () =>
        await alice.call(dao, 'add_proposal', {
            proposal: {
                description: 'change the policy',
                kind: {
                    ChangePolicy: {
                       policy
                    }
                }
            },
        },
            { attachedDeposit: toYocto('1') }
        )
    );
    test.regex(errorString, /ERR_INVALID_POLICY/); 

    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    
    const correctPolicy = 
    {
        roles: [
            {
                name: "all",
                kind: { "Group": [alice.accountId] },
                permissions: ["*:AddProposal",
                    "*:VoteApprove"],
                vote_policy: {}
            }
        ],
        default_vote_policy:
        {
            weight_kind: "TokenWeight",
            quorum: new BN('1').toString(),
            threshold: '5',
        },
        proposal_bond: toYocto('1'),
        proposal_period: period,
        bounty_bond: toYocto('1'),
        bounty_forgiveness_period: period,
    };
    let id: any = await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'change to a new correct policy',
            kind: {
                ChangePolicy: {
                    policy: correctPolicy
                }
            }
        },
    },
        { attachedDeposit: toYocto('1') }
    )
    await voteApprove(root, dao, id);
});

workspace.test('Proposal SetStakingContract', async (test, {alice, root, dao})=>{
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    await setStakingId(root, dao, staking);

    let errorString = await captureError(async () =>
        await setStakingId(root, dao, staking)
    );
    test.regex(errorString, /ERR_STAKING_CONTRACT_CANT_CHANGE/); 
});
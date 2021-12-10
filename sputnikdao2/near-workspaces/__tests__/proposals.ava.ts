import { toYocto, NearAccount, captureError, BN } from 'near-workspaces-ava';

import { workspace } from './utils';

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
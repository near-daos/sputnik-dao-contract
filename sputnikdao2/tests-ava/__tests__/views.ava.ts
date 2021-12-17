import { Workspace, BN, NearAccount, captureError, toYocto, tGas, ONE_NEAR } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';

const DEADLINE = '1925376849430593581';
const BOND = toYocto('1');

async function proposeBounty(alice: NearAccount, dao: NearAccount) {  
    const bounty = {
        description: 'test_bounties',
        //token: alice,
        amount: '19000000000000000000000000',
        times: 3,
        max_deadline: DEADLINE
    }
    const proposalId: number = await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'add_new_bounty',
            kind: {
                AddBounty: {
                    bounty
                }
            }
        },
    },
        { 
            attachedDeposit: toYocto('1') 
        }
    )
    return proposalId;
}

async function voteOnBounty(root: NearAccount, dao: NearAccount, proposalId: number) {
    await root.call(dao, 'act_proposal', 
    {
        id: proposalId,
        action: 'VoteApprove'
    })
}

async function claimBounty(alice: NearAccount, dao: NearAccount, proposalId: number) {
    await alice.call(dao, 'bounty_claim', 
    {
        id: proposalId,
        deadline: DEADLINE

    },
    { 
        attachedDeposit: BOND
    })
}

workspace.test('View version', async (test, {alice, root, dao }) => {
    test.log(await dao.view('version'));
});

workspace.test('View get_config', async (test, {root}) => {
    const config = { name: 'sputnikda2', purpose: 'testing get_config', metadata: '' }
    const policy = [root.accountId]

    const bob = await root.createAndDeploy(
        'bob',
        '../res/sputnikdao2.wasm',
        {
            method: 'new',
            args: { config, policy },
            initialBalance: toYocto('200'),
        }
    );
    test.deepEqual(await bob.view('get_config'), config)
});

workspace.test('View get_policy', async (test, {root }) => {
    const config = { name: 'sputnikda2', purpose: 'testing get_policy', metadata: '' }
    const versionedPolicy = [root.accountId]

    const bob = await root.createAndDeploy(
        'bob',
        '../res/sputnikdao2.wasm',
        {
            method: 'new',
            args: { config, policy: versionedPolicy },
            initialBalance: toYocto('200'),
        }
    );
    const policy = {
        roles: [
            {
                name: 'all',
                kind: 'Everyone',
                permissions: ['*:AddProposal'],
                vote_policy: {}
            },
            {
                name: 'council',
                kind: {
                    Group: [root.accountId]
                },
                permissions: [
                    '*:Finalize',
                    '*:AddProposal',
                    '*:VoteApprove',
                    '*:VoteReject',
                    '*:VoteRemove'
                ],
                vote_policy: {}
            }
        ],
        default_vote_policy: {
            weight_kind: 'RoleWeight',
            quorum: '0',
            threshold: [1, 2]
        },
        proposal_bond: '1000000000000000000000000',
        proposal_period: '604800000000000',
        bounty_bond: '1000000000000000000000000',
        bounty_forgiveness_period: '86400000000000'
    };
    test.deepEqual(await bob.view('get_policy'), policy)
});

workspace.test('View get_staking_contract', async (test, {alice, root, dao }) => {
    test.is(await dao.view('get_staking_contract'), null);
});

workspace.test('View has_blob', async (test, {alice, root, dao }) => {
    //console.log(await dao.view('has_blob', {some_}));
});

workspace.test('View get_locked_storage_amount', async (test, {alice, root, dao }) => {
});

workspace.test('View get_available_amount', async (test, {alice, root, dao }) => {
});

workspace.test('View delegation_total_supply', async (test, {alice, root, dao }) => {
});

workspace.test('View delegation_balance_of', async (test, {alice, root, dao }) => {
});

workspace.test('View delegation_balance_ratio', async (test, {alice, root, dao }) => {
});

workspace.test('View methods for proposals', async (test, {alice, root, dao }) => {
    //Test get_last_proposal_id
    test.is(await dao.view('get_last_proposal_id'), 0);

    //Test get_proposals
    test.deepEqual(await dao.view('get_proposals', {from_index: 0, limit: 100}), []);

    const config = {
        name: 'sputnikdao2',
        purpose: 'testing_view_methods',
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

    const realProposalAlice = {
        id: 0,
        proposer: alice.accountId,
        description: 'rename the dao',
        kind: {ChangeConfig: {config}},
        status: 'InProgress',
        vote_counts: {},
        votes: {}
    };

    const proposalAlice: any = await dao.view('get_proposal', {id: 0});

    //Test get_proposal
    test.is(proposalAlice.proposer, realProposalAlice.proposer);
    test.is(proposalAlice.description, realProposalAlice.description);
    test.is(proposalAlice.status, realProposalAlice.status);
    test.deepEqual(proposalAlice.vote_counts, realProposalAlice.vote_counts);
    test.deepEqual(proposalAlice.votes, realProposalAlice.votes);
    test.deepEqual(proposalAlice.kind, realProposalAlice.kind);

    //Test get_last_proposal_id
    test.deepEqual(await dao.view('get_last_proposal_id'), 1);

    //Test get_proposals
    const proposals: any = await dao.view('get_proposals', {from_index: 0, limit: 100});
    test.is(proposals[0].proposer, realProposalAlice.proposer);
    test.is(proposals[0].description, realProposalAlice.description);
    test.is(proposals[0].status, realProposalAlice.status);
    test.deepEqual(proposals[0].vote_counts, realProposalAlice.vote_counts);
    test.deepEqual(proposals[0].votes, realProposalAlice.votes);
    test.deepEqual(proposals[0].kind, realProposalAlice.kind);

});

workspace.test('View methods for bounties', async (test, {alice, root, dao }) => {
    //Test get_last_bounty_id
    test.is(await dao.view('get_last_bounty_id'), 0);
    //Test get_bounties
    test.deepEqual(await dao.view('get_bounties', {from_index: 0, limit: 100}), []);

    const proposalId = await proposeBounty(alice, dao);
    const bounty = {
        id: 0,
        description: 'test_bounties',
        token: null,
        amount: '19000000000000000000000000',
        times: 3,
        max_deadline: DEADLINE    
    }
    await voteOnBounty(root, dao, proposalId);

    //Test get_last_bounty_id
    test.is(await dao.view('get_last_bounty_id'), 1);
    //Test get_bounties
    test.deepEqual(await dao.view('get_bounties', {from_index: 0, limit: 100}), [bounty]);
    //Test get_bounty
    test.deepEqual(await dao.view('get_bounty', {id: 0}), bounty);

    await claimBounty(alice, dao, proposalId);

    //Test get_bounty_number_of_claims
    test.is(await dao.view('get_bounty_number_of_claims', {id: 0}), 1);
    //Test get_bounty_claims
    const realClaim = {
        bounty_id: 0,
        deadline: DEADLINE,
        completed: false
    };
    const claim: any = await dao.view('get_bounty_claims', {account_id: alice.accountId});
    //test.is(claim.bounty_id, realClaim.bounty_id);
    test.is(claim.deadline, realClaim.deadline);
    test.is(claim.completed, realClaim.completed);
});
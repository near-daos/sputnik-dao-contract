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

async function doneBounty(alice: NearAccount, bob: NearAccount, dao: NearAccount, proposalId: number) {
    await alice.call(dao, 'bounty_done', 
    {
        id: proposalId,
        account_id: bob,
        description: 'This bounty is done'

    },
    { 
        attachedDeposit: toYocto('1') 
    })
}

async function giveupBounty(alice: NearAccount, dao: NearAccount, proposalId: number) {
    await alice.call(dao, 'bounty_giveup', { id: proposalId })
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
});

workspace.test('View has_blob', async (test, {alice, root, dao }) => {
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

workspace.test('View get_last_proposal_id', async (test, {alice, root, dao }) => {
});

workspace.test('View get_proposals', async (test, {alice, root, dao }) => {
});

workspace.test('View get_proposal', async (test, {alice, root, dao }) => {
});

workspace.test('View get_last_bounty_id', async (test, {alice, root, dao }) => {
});

workspace.test('View get_bounty', async (test, {alice, root, dao }) => {
});

workspace.test('View get_bounties', async (test, {alice, root, dao }) => {
});

workspace.test('View get_bounty_claims', async (test, {alice, root, dao }) => {
});

workspace.test('View get_bounty_number_of_claims', async (test, {alice, root, dao }) => {
});

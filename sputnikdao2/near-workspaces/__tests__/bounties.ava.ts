import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';

async function proposeBounty(alice: NearAccount, dao: NearAccount) {  
    const bounty = {
        description: 'test_bounties',
        token: alice,
        amount: '1',
        times: 3,
        max_deadline: '1925376849430593581'
    }
    await alice.call(dao, 'add_proposal', {
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
}

async function voteOnBounty(alice: NearAccount, dao: NearAccount) {
    await alice.call(dao, 'act_proposal', 
    {
        id: 0,
        action: 'VoteApprove'
    })
}

async function claimBounty(alice: NearAccount, dao: NearAccount) {
    await alice.call(dao, 'act_proposal', 
    {
        id: 0,
        deadline: '1925376849430593581'

    },
    { 
        attachedDeposit: toYocto('1') 
    })
}

workspace.test('Bounty claim', async (test, {alice, root, dao }) => {
    proposeBounty(alice, dao);
    voteOnBounty(alice, dao);
    claimBounty(alice, dao);
});

workspace.test('Bounty done', async (test, {alice, root, dao }) => {
    const regCost = STORAGE_PER_BYTE.mul(new BN(16));

    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);

    proposeBounty(alice, dao);

    let errorString = await captureError(async () =>
        staking.call(dao, 'bounty_done', 
        {
            id: 1,
            account_id: alice,
            description: 'new_bounty_done'
        },
            { attachedDeposit: regCost }));
    //test.regex(errorString, /ERR_BOUNTY_DONE_MUST_BE_SELF/);
});

workspace.test('Bounty giveup', async (test, {alice, root, dao }) => {
    proposeBounty(alice, dao);
});
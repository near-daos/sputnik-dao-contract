import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
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

async function doneBounty(alice: NearAccount, dao: NearAccount, proposalId: number) {
    await alice.call(dao, 'bounty_done', 
    {
        id: proposalId,
        description: 'This bounty is done'

    },
    { 
        attachedDeposit: toYocto('1') 
    })
}


workspace.test('Bounty workflow', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    console.log('Before bounty_done:');
    console.log(await dao.view('get_bounty_claims', { account_id: alice }));
    await doneBounty(alice, dao, proposalId);
    console.log('After bounty_done:');
    console.log(await dao.view('get_bounty_claims', { account_id: alice }));
    console.log('Before act_proposal, voting on the bounty:')
    console.log(await dao.view('get_proposal', { id: proposalId + 1 }));
    await voteOnBounty(root, dao, proposalId + 1);
    console.log('After act_proposal, voting on the bounty:')
    console.log(await dao.view('get_proposal', { id: proposalId + 1 }));
    
});

workspace.test('Bounty claim', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);

    //The method chould panic if the bounty with given id doesn't exist
    let errorString1 = await captureError(async () =>
        await claimBounty(alice, dao, proposalId)
    );
    test.regex(errorString1, /ERR_NO_BOUNTY/);

    await voteOnBounty(root, dao, proposalId);

    //Should panic if `attached_deposit` 
    //is not equal to the corresponding `bounty_bond`
    //If we attach more than needed:
    let errorString2_1 = await captureError(async () =>
        await alice.call(dao, 'bounty_claim', 
        {
            id: proposalId,
            deadline: DEADLINE
        },
        { 
            attachedDeposit: new BN(BOND).addn(1)
        })
    );
    test.regex(errorString2_1, /ERR_BOUNTY_WRONG_BOND/);
    //If we attach less than needed:
    let errorString2_2 = await captureError(async () =>
        await alice.call(dao, 'bounty_claim', 
        {
            id: proposalId,
            deadline: DEADLINE
        },
        { 
            attachedDeposit: new BN(BOND).subn(1)
        })
    );
    test.regex(errorString2_2, /ERR_BOUNTY_WRONG_BOND/);

    //Should panic in case of wrong deadline
    let errorString3 = await captureError(async () =>
        await alice.call(dao, 'bounty_claim', 
        {
            id: proposalId,
            deadline: '1925376849430593582'
        },
        { 
            attachedDeposit: BOND
        })
    );
    test.regex(errorString3, /ERR_BOUNTY_WRONG_DEADLINE/);

    await claimBounty(alice, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    
    //Should panic if all bounties are claimed
    let errorString4 = await captureError(async () =>
        await alice.call(dao, 'bounty_claim', 
        {
            id: proposalId,
            deadline: '1925376849430593582'
        },
        { 
            attachedDeposit: toYocto('1') 
        })
    );
    test.regex(errorString4, /ERR_BOUNTY_ALL_CLAIMED/);
});

workspace.test('Bounty done', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
});

workspace.test('Bounty giveup', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
});
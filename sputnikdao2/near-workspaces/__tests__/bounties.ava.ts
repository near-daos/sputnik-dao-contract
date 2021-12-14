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

workspace.test('Bounty workflow', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    console.log('Claims before bounty_done:');
    console.log(await dao.view('get_bounty_claims', { account_id: alice }));
    await doneBounty(alice, alice, dao, proposalId);
    console.log('Claims after bounty_done:');
    console.log(await dao.view('get_bounty_claims', { account_id: alice }));
    console.log('The proposal before act_proposal, voting on the bounty:')
    console.log(await dao.view('get_proposal', { id: proposalId + 1 }));
    await voteOnBounty(root, dao, proposalId + 1);
    console.log('The proposal after act_proposal, voting on the bounty:')
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
            deadline: DEADLINE
        },
        { 
            attachedDeposit: BOND
        })
    );
    test.regex(errorString4, /ERR_BOUNTY_ALL_CLAIMED/);
});

workspace.test('Bounty done', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);

    const bob = await root.createAccount('bob');

    //Should panic if the caller is not in the list of claimers
    let errorString1 = await captureError(async () =>
        await doneBounty(alice, bob, dao, proposalId)
    );
    test.regex(errorString1, /ERR_NO_BOUNTY_CLAIMS/);

    await claimBounty(bob, dao, proposalId);

    //Should panic if the list of claims for the caller of the method 
    //doesn't contain the claim with given ID
    let errorString2 = await captureError(async () =>
        await doneBounty(alice, alice, dao, proposalId + 10)
    );
    test.regex(errorString2, /ERR_NO_BOUNTY_CLAIM/);


    //`bounty_done` can only be called by the claimer
    let errorString3 = await captureError(async () =>
        await doneBounty(alice, bob, dao, proposalId)
    );
    test.regex(errorString3, /ERR_BOUNTY_DONE_MUST_BE_SELF/);

    await doneBounty(alice, alice, dao, proposalId);
    await voteOnBounty(root, dao, proposalId + 1);


    //Should panic if the bounty claim is completed
    let errorString4 = await captureError(async () =>
        await doneBounty(alice, alice, dao, proposalId)
    );
    test.regex(errorString4, /ERR_BOUNTY_CLAIM_COMPLETED/);

});

workspace.test('Bounty giveup', async (test, {alice, root, dao }) => {
    const proposalId = await proposeBounty(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);

    //Should panic if the caller is not in the list of claimers
    const bob = await root.createAccount('bob');
    let errorString1 = await captureError(async () =>
        await giveupBounty(bob, dao, proposalId)
    );
    test.regex(errorString1, /ERR_NO_BOUNTY_CLAIMS/);

    //Should panic if the list of claims for the caller of the method 
    //doesn't contain the claim with given ID
    let errorString2 = await captureError(async () =>
        await giveupBounty(alice, dao, proposalId + 10)
    );
    test.regex(errorString2, /ERR_NO_BOUNTY_CLAIM/);
});
import { Workspace, BN, NearAccount, captureError, toYocto, tGas, ONE_NEAR, NEAR } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';
import { DEADLINE, BOND, proposeBounty, proposeBountyWithNear, voteOnBounty, claimBounty, doneBounty, giveupBounty, voteApprove } from './utils'

workspace.test('Bounty workflow', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const proposalId = await proposeBounty(alice, dao, testToken);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    const proposal = await dao.view('get_bounty_claims', { account_id: alice })
    test.log('Claims before bounty_done:');
    test.log(await dao.view('get_bounty_claims', { account_id: alice }));
    await doneBounty(alice, alice, dao, proposalId);
    test.log('Claims after bounty_done:');
    test.log(await dao.view('get_bounty_claims', { account_id: alice }));
    test.log('The proposal before act_proposal, voting on the bounty:')
    test.log(await dao.view('get_proposal', { id: proposalId + 1 }));
    await voteOnBounty(root, dao, proposalId + 1);
    test.log('The proposal after act_proposal, voting on the bounty:')
    test.log(await dao.view('get_proposal', { id: proposalId + 1 }));

});

workspace.test('Bounty claim', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const proposalId = await proposeBounty(alice, dao, testToken);

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

    //Should increase number of claims
    test.is(await dao.view('get_bounty_number_of_claims', { id: proposalId }), 1);

    //Should add this claim to the list of claims, done by this account
    let bounty: any = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty[0].bounty_id, 0);
    test.is(bounty[0].deadline, DEADLINE);
    test.is(bounty[0].completed, false);


    await claimBounty(alice, dao, proposalId);
    test.is(await dao.view('get_bounty_number_of_claims', { id: proposalId }), 2);

    let bounty2: any = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty2[1].bounty_id, 0);
    test.is(bounty2[1].deadline, DEADLINE);
    test.is(bounty2[1].completed, false);


    await claimBounty(alice, dao, proposalId);
    test.is(await dao.view('get_bounty_number_of_claims', { id: proposalId }), 3);

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

/*
workspace.test('Bounty done', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const proposalId = await proposeBounty(alice, dao, testToken);
    await dao.call(testToken, 'mint',
        {
            account_id: dao,
            amount: '1000000000',
        },
        {
            gas: tGas(200)
        }
    );
    await dao.call(testToken, 'mint',
        {
            account_id: root,
            amount: '1000000000',
        },
        {
            gas: tGas(200)
        }
    );
    await dao.call(testToken, 'mint',
        {
            account_id: alice,
            amount: '1000000000',
        },
        {
            gas: tGas(200)
        }
    );

    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);

    const bob = await root.createAccount('bob');
    await bob.call(testToken, 'storage_deposit', 
    {
        account_id: bob.accountId,
        registration_only: true,
    },
    {
        attachedDeposit: toYocto('50'),
    }
    );
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

    let bounty: any = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty[0].completed, false);

    console.log(await dao.view('get_proposals', { from_index: 0, limit: 10 }));

    const whale = await root.createAccount('whale');
    await whale.call(testToken, 'mint',
        {
            account_id: whale.accountId,
            amount: '6000000000',
        },
        {
            gas: tGas(200)
        }
    );
    await whale.call(testToken, 'ft_transfer',
        {
            receiver_id: dao.accountId,
            amount: '10000',
            memo: 'Heard you are in a pinch, let me help.'
        },
        {
            attachedDeposit: new BN(1)
        }
    );
    

    await doneBounty(alice, alice, dao, proposalId);

    //claim is marked as completed
    bounty = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty[0].completed, true);

    let proposal: any = await dao.view('get_proposal', { id: proposalId + 1 });
    test.is(proposal.status, 'InProgress');

    console.log(await dao.view('get_proposals', { from_index: 0, limit: 10 }));

    await voteOnBounty(root, dao, proposalId + 1);

    console.log(await dao.view('get_proposals', { from_index: 0, limit: 10 }));

    //proposal is approved
    proposal = await dao.view('get_proposal', { id: proposalId + 1 });
    test.is(proposal.status, 'Approved');


    //Should panic if the bounty claim is completed
    let errorString4 = await captureError(async () =>
        await doneBounty(alice, alice, dao, proposalId)
    );
    test.regex(errorString4, /ERR_BOUNTY_CLAIM_COMPLETED/);

});*/

workspace.test('Bounty done with NEAR token', async (test, { alice, root, dao }) => {
    const proposalId = await proposeBountyWithNear(alice, dao);
    
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

    let bounty: any = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty[0].completed, false);

    await doneBounty(alice, alice, dao, proposalId);

    //claim is marked as completed
    bounty = await dao.view('get_bounty_claims', { account_id: alice });
    test.is(bounty[0].completed, true);

    let proposal: any = await dao.view('get_proposal', { id: proposalId + 1 });
    test.is(proposal.status, 'InProgress');

    await voteOnBounty(root, dao, proposalId + 1);

    //proposal is approved
    proposal = await dao.view('get_proposal', { id: proposalId + 1 });
    test.is(proposal.status, 'Approved');

    //Should panic if the bounty claim is completed
    let errorString4 = await captureError(async () =>
        await doneBounty(alice, alice, dao, proposalId)
    );
    test.regex(errorString4, /ERR_BOUNTY_CLAIM_COMPLETED/);

});

workspace.test('Bounty giveup', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const proposalId = await proposeBounty(alice, dao, testToken);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);

    //Should panic if the caller is not in the list of claimers
    const bob = await root.createAccount('bob');
    let errorString = await captureError(async () =>
        await giveupBounty(bob, dao, proposalId)
    );
    test.regex(errorString, /ERR_NO_BOUNTY_CLAIMS/);

    //Should panic if the list of claims for the caller of the method 
    //doesn't contain the claim with given ID
    errorString = await captureError(async () =>
        await giveupBounty(alice, dao, proposalId + 10)
    );
    test.regex(errorString, /ERR_NO_BOUNTY_CLAIM/);

    //If within forgiveness period, `bounty_bond` should be returned ???
    const balance1: NEAR = (await alice.balance()).total;
    const result = await giveupBounty(alice, dao, proposalId);
    const balance2: NEAR = (await alice.balance()).total;
    test.is(
        Number(balance2.add(result.gas_burnt).toHuman().slice(0, -1)).toFixed(1),
        Number(balance1.add(ONE_NEAR).toHuman().slice(0, -1)).toFixed(1)
    );
    test.not(balance2, balance1);

    //If within forgiveness period, 
    //claim should be removed from the list of claims, done by this account
    test.deepEqual(await dao.view('get_bounty_claims', { account_id: alice }), []);
});

workspace.test('Bounty ft done', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    let proposalId = await proposeBounty(alice, dao, testToken);
    await dao.call(
        testToken,
        'mint',
        {
            account_id: dao,
            amount: '1000000000',
        },
        {
            gas: tGas(50)
        }
    );
    await alice.call(
        testToken,
        'storage_deposit',
        {
            account_id: alice.accountId,
            registration_only: true,
        },
        {
            attachedDeposit: toYocto('90'),
        }
    );
    const bounty = {
        description: 'test_bounties',
        token: testToken.accountId,
        amount: '10',
        times: 3,
        max_deadline: DEADLINE
    }
    proposalId = await alice.call(dao, 'add_proposal', {
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
    );
    await voteApprove(root, dao, proposalId);
    let { status } = await dao.view('get_proposal', { id: proposalId });
    test.is(status, 'Approved');
    const bountyId = 0; // first bounty
    await claimBounty(alice, dao, bountyId);
    await alice.call(dao, 'bounty_done',
    {
        id: bountyId,
        account_id: alice.accountId,
        description: 'This bounty is done'

    },
    {
        attachedDeposit: toYocto('1')
    });

    await voteApprove(root, dao, proposalId + 1);
    ({ status } = await dao.view('get_proposal', { id: proposalId }));
    test.is(status, 'Approved');
})
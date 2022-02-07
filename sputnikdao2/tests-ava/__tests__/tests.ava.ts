import { Workspace, BN, NearAccount, captureError, toYocto, tGas, ONE_NEAR, NEAR } from 'near-workspaces-ava';
import * as fs from 'fs';

const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');

async function initWorkspace(root: NearAccount) {

    const alice = await root.createAccount('alice');
    // console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N

    const config = { name: 'sputnik', purpose: 'testing', metadata: '' }
    const policy = [root.accountId]

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            method: 'new',
            args: { config, policy },
            initialBalance: toYocto('200'),
        }
    );

    // console.log('dao\'s balance is: ' + (await dao.balance()).total) //~200N

    return { alice, dao };
}

export const STORAGE_PER_BYTE = new BN('10000000000000000000');

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root)
});

export const workspaceWithoutInit = Workspace.init(async ({ root }) => {
    const alice = await root.createAccount('alice');

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            initialBalance: toYocto('200'),
        }
    );
    return { alice, dao };
});

export async function initTestToken(root: NearAccount) {
    const testToken = await root.createAndDeploy(
        'test-token',
        '../../test-token/res/test_token.wasm',
        {
            method: 'new',
            initialBalance: toYocto('200'),
        }
    );
    return testToken;
}

export async function initStaking(root: NearAccount, dao: NearAccount, testToken: NearAccount) {
    const staking = await root.createAndDeploy(
        'staking',
        '../../sputnik-staking/res/sputnik_staking.wasm',
        {
            method: 'new',
            args: { owner_id: dao, token_id: testToken, unstake_period: '100000000000' },
            initialBalance: toYocto('100'),
        }
    );
    return staking
}

export async function setStakingId(root: NearAccount, dao: NearAccount, staking: NearAccount) {
    // Setting staking id
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal:
            {
                description: 'test',
                kind: { "SetStakingContract": { "staking_id": staking.accountId } }
            },
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
}

export const regCost = STORAGE_PER_BYTE.mul(new BN(16));

export async function registerAndDelegate(dao: NearAccount, staking: NearAccount, account: NearAccount, amount: BN) {
    await staking.call(dao, 'register_delegation', { account_id: account },
        { attachedDeposit: regCost }
    );
    const res: string[3] = await staking.call(
        dao,
        'delegate',
        {
            account_id: account,
            amount: amount.toString(),
        }
    );
    return res;
}

export const DEADLINE = '1925376849430593581';
export const BOND = toYocto('1');

export async function proposeBounty(alice: NearAccount, dao: NearAccount, token: NearAccount) {
    const bounty = {
        description: 'test_bounties',
        token: token.accountId,
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

export async function proposeBountyWithNear(alice: NearAccount, dao: NearAccount) {
    const bounty = {
        description: 'test_bounties_with_near_token',
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

export async function voteOnBounty(root: NearAccount, dao: NearAccount, proposalId: number) {
    await root.call(dao, 'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove'
        },
        {
            gas: tGas(50)
        })
}

export async function claimBounty(alice: NearAccount, dao: NearAccount, proposalId: number) {
    await alice.call(dao, 'bounty_claim',
        {
            id: proposalId,
            deadline: DEADLINE

        },
        {
            attachedDeposit: BOND
        })
}

export async function doneBounty(alice: NearAccount, bob: NearAccount, dao: NearAccount, proposalId: number) {
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

export async function giveupBounty(alice: NearAccount, dao: NearAccount, proposalId: number) {
    return await alice.call_raw(dao, 'bounty_giveup', { id: proposalId })
}

export async function voteApprove(root: NearAccount, dao: NearAccount, proposalId: number) {
    await root.call(dao, 'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove'
        },
        {
            gas: tGas(100),
        })
}

// bounties tests
// -------------------------------------------------------------------------------------------
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
    test.regex(errorString4, /ERR_NO_BOUNTY_CLAIMS/);

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
    let proposalId: number = await alice.call(dao, 'add_proposal', {
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

// delegation tests
// ---------------------------------------------------------------------------------------------
workspace.test('Register delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);

    // set staking
    await setStakingId(root, dao, staking);

    await registerAndDelegate(dao, staking, alice, new BN(1));

    // Check that delegation appears in `delegations` LookupMap.
    let bal: BN = new BN(await dao.view('delegation_balance_of', { account_id: alice }));
    test.deepEqual(bal, new BN(1));
    const total: BN = new BN(await dao.view('delegation_total_supply'));
    test.deepEqual(total, new BN(1));
});


workspace.test('Register delegation fail', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);

    // Staking id not set 
    let errorString = await captureError(async () =>
        staking.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost }));
    test.regex(errorString, /ERR_NO_STAKING/);

    await setStakingId(root, dao, staking);
    // Can only be called by the `staking_id`
    errorString = await captureError(async () =>
        root.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost }));
    test.regex(errorString, /ERR_INVALID_CALLER/);

    // Attached deposit is handled correctly
    await captureError(async () =>
        root.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost.add(new BN(1)) }));
    await captureError(async () =>
        root.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost.sub(new BN(1)) }));
});


workspace.test('Delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');
    const bob = await root.createAccount('bob');

    // set staking
    await setStakingId(root, dao, staking);

    let result = await registerAndDelegate(dao, staking, alice, randomAmount);
    test.deepEqual([new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [new BN('0'), randomAmount, randomAmount]);
    result = await registerAndDelegate(dao, staking, bob, randomAmount.muln(2));
    test.deepEqual([new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [new BN('0'), randomAmount.muln(2), randomAmount.muln(3)]);
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: alice })),
        randomAmount);
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: bob })),
        randomAmount.muln(2));
    test.deepEqual(new BN(
        await dao.view('delegation_total_supply')),
        randomAmount.muln(3));
});


workspace.test('Delegation fail', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(
            dao,
            'delegate',
            {
                account_id: alice,
                amount: randomAmount,
            })
    );
    test.regex(errorString, /ERR_NO_STAKING/);

    // set staking
    await setStakingId(root, dao, staking);

    // Check that it can only be called by the `staking_id`
    errorString = await captureError(async () =>
        root.call(
            dao,
            'delegate',
            {
                account_id: alice,
                amount: randomAmount,
            })
    );
    test.regex(errorString, /ERR_INVALID_CALLER/);

    // Can't be called without previos registration
    errorString = await captureError(async () =>
        staking.call(
            dao,
            'delegate',
            {
                account_id: 'not-registered-account.bob',
                amount: randomAmount,
            })
    );
    test.regex(errorString, /ERR_NOT_REGISTERED/);
});


workspace.test('Undelegate', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('44887687667868');

    // set staking
    await setStakingId(root, dao, staking);

    await registerAndDelegate(dao, staking, alice, randomAmount);

    // Check that amount is subtracted correctly
    const result: string[3] = await staking.call(
        dao,
        'undelegate',
        {
            account_id: alice,
            amount: randomAmount.divn(2).toString(),
        }
    );
    test.deepEqual([new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [randomAmount, randomAmount.divn(2), randomAmount.divn(2)]);
});


workspace.test('Undelegate fail', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('44887687667868');

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(
            dao,
            'undelegate',
            {
                account_id: alice,
                amount: randomAmount,
            })
    );
    test.regex(errorString, /ERR_NO_STAKING/);

    // Set staking
    await setStakingId(root, dao, staking);

    // Check that it can only be called by the `staking_id`
    errorString = await captureError(async () =>
        root.call(
            dao,
            'undelegate',
            {
                account_id: alice,
                amount: randomAmount,
            })
    );
    test.regex(errorString, /ERR_INVALID_CALLER/);

    await registerAndDelegate(dao, staking, alice, randomAmount);
    // Check that a user can't remove more than it delegated
    errorString = await captureError(async () =>
        staking.call(
            dao,
            'undelegate',
            {
                account_id: alice,
                amount: randomAmount.addn(1).toString(),
            })
    );
    test.regex(errorString, /ERR_INVALID_STAKING_CONTRACT/);
});

// lib tests
// ------------------------------------------------------------------------------------------

workspace.test('Upgrade self', async (test, { root, dao }) => {
    const result = await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();
    const hash = result.parseResult<String>()
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal:
            {
                description: 'test',
                kind: { "UpgradeSelf": { hash: hash } }
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );


    const id: number = await dao.view('get_last_proposal_id');
    test.is(id, 1);

    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(300), // attempt to subtract with overflow if not enough gas, maybe add some checks?
        }
    );

    test.is(await dao.view('version'), "2.0.0");

    const beforeBlobRemove = new BN(await dao.view('get_available_amount'));
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
    test.assert(
        new BN(await dao.view('get_available_amount')).gt(beforeBlobRemove)
    )
});

workspaceWithoutInit.test('Upgrade self negative', async (test, { root, dao }) => {
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };

    // NOT INITIALIZED
    let err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('200'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_CONTRACT_IS_NOT_INITIALIZED/);

    // Initializing contract
    await root.call(
        dao,
        'new',
        { config, policy: [root.accountId] },
    );

    // not enough deposit
    err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('1'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_NOT_ENOUGH_DEPOSIT/);

    await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();

    // Already exists
    err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('200'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_ALREADY_EXISTS/);

});

workspace.test('Remove blob', async (test, { root, dao, alice }) => {
    const result = await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();

    const hash = result.parseResult<String>()
    
    // fails if hash is wrong
    let err = await captureError(async () =>
        root.call(
            dao,
            'remove_blob',
            {
                hash: "HLBiX51txizmQzZJMrHMCq4u7iEEqNbaJppZ84yW7628", // some_random hash
            }
        )
    );
    test.regex(err, /ERR_NO_BLOB/);

    // Can only be called by the original storer
    err = await captureError(async () =>
        alice.call(
            dao,
            'remove_blob',
            {
                hash: hash,
            }
        )
    );
    test.regex(err, /ERR_INVALID_CALLER/);

    // blob is removed with payback
    const rootAmountBeforeRemove = (await root.balance()).total
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
    const rootAmountAfterRemove = (await root.balance()).total
    test.false(await dao.view('has_blob', { hash: hash }));
    test.assert(rootAmountAfterRemove.gt(rootAmountBeforeRemove));
});

workspace.test('Callback for BountyDone with NEAR token', async (test, { alice, root, dao }) => {
    //During the callback the number bounty_claims_count should decrease
    const proposalId = await proposeBountyWithNear(alice, dao);
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    await doneBounty(alice, alice, dao, proposalId);
    //Before the bounty is done there is 1 claim
    test.is(await dao.view('get_bounty_number_of_claims', {id: 0}), 1);
    const balanceBefore: NEAR = (await alice.balance()).total;
    //During the callback this number is decreased
    await voteOnBounty(root, dao, proposalId + 1);
    const balanceAfter: NEAR = (await alice.balance()).total;
    test.is(await dao.view('get_bounty_number_of_claims', {id: 0}), 0);
    test.assert(balanceBefore.lt(balanceAfter));
});

workspace.test('Callback for BountyDone ft token fail', async (test, { alice, root, dao }) => {
    //Test the callback with Failed proposal status
    const testTokenFail = await initTestToken(root);
    const proposalIdFail = await proposeBounty(alice, dao, testTokenFail);
    await dao.call(
        testTokenFail,
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
        testTokenFail,
        'storage_deposit',
        {
            account_id: alice.accountId,
            registration_only: true,
        },
        {
            attachedDeposit: toYocto('90'),
        }
    );
    await voteOnBounty(root, dao, proposalIdFail);
    await claimBounty(alice, dao, proposalIdFail);
    await doneBounty(alice, alice, dao, proposalIdFail);
    await voteOnBounty(root, dao, proposalIdFail + 1);
    //Proposal should be Failed
    let { status } = await dao.view('get_proposal', { id: proposalIdFail + 1 });
    test.is(status, 'Failed');
});
    
workspace.test('Callback for BountyDone ft token', async (test, { alice, root, dao }) => {
    //Test correct callback
    const testToken = await initTestToken(root);
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
    let proposalId: number = await alice.call(dao, 'add_proposal', {
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
    await voteOnBounty(root, dao, proposalId);
    await claimBounty(alice, dao, proposalId);
    await doneBounty(alice, alice, dao, proposalId);
    //Before the bounty is done there is 1 claim
    test.is(await dao.view('get_bounty_number_of_claims', {id: 0}), 1);
    const balanceBefore: NEAR = (await alice.balance()).total;
    //During the callback this number is decreased
    await voteOnBounty(root, dao, proposalId + 1);
    
    //Proposal should be approved
    let { status } = await dao.view('get_proposal', { id: proposalId + 1 });
    test.is(status, 'Approved');

    //During the callback the number bounty_claims_count should decrease
    const balanceAfter: NEAR = (await alice.balance()).total;
    test.is(await dao.view('get_bounty_number_of_claims', {id: 0}), 0);
    test.assert(balanceBefore.lt(balanceAfter));
});

workspace.test('Callback transfer', async (test, { alice, root, dao }) => {
    const user1 = await root.createAccount('user1');
    // Fail transfer by transfering to non-existent accountId
    let transferId: number = await user1.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'give me tokens',
            kind: {
                Transfer: {
                    token_id: "",
                    receiver_id: "broken_id",
                    amount: toYocto('1'),
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });
    let user1Balance = (await user1.balance()).total
    await voteApprove(root, dao, transferId);
    let { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Failed');
    test.assert((await user1.balance()).total.eq(user1Balance)); // no bond returns on fail

    // now we transfer to real accountId
    transferId = await user1.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'give me tokens',
            kind: {
                Transfer: {
                    token_id: "",
                    receiver_id: alice.accountId, // valid id this time
                    amount: toYocto('1'),
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });
    user1Balance = (await user1.balance()).total
    await voteApprove(root, dao, transferId);
    ({ status } = await dao.view('get_proposal', { id: transferId }));
    test.is(status, 'Approved');
    test.assert((await user1.balance()).total.gt(user1Balance)); // returns bond
});

workspace.test('Callback function call', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    let transferId: number = await root.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'give me tokens',
            kind: {
                FunctionCall: {
                    receiver_id: testToken.accountId,
                    actions: [{ method_name: 'fail', args: Buffer.from('bad args').toString('base64'), deposit: toYocto('1'), gas: tGas(10) }],
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });
    await root.call(dao, 'act_proposal',
        {
            id: transferId,
            action: 'VoteApprove'
        },
        {
            gas: tGas(200),
        });
    let { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Failed');

    transferId = await root.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'give me tokens',
            kind: {
                FunctionCall: {
                    receiver_id: testToken.accountId,
                    actions: [
                        { method_name: 'mint', args: Buffer.from('{"account_id": "' + alice.accountId + '", "amount": "10"}').toString('base64'), deposit: '0', gas: tGas(10) },
                        { method_name: 'burn', args: Buffer.from('{"account_id": "' + alice.accountId + '", "amount": "10"}').toString('base64'), deposit: '0', gas: tGas(10) }],
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });
    await root.call(dao, 'act_proposal',
        {
            id: transferId,
            action: 'VoteApprove'
        },
        {
            gas: tGas(200),
        });
    ({ status } = await dao.view('get_proposal', { id: transferId }));
    test.is(status, 'Approved');
});

// policy tests
// ---------------------------------------------------------------------------------------------

workspaceWithoutInit.test('Testing policy TokenWeight', async (test, { alice, root, dao }) => {
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
    const bob = await root.createAccount('bob')
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    await root.call(
        dao,
        'new',
        { config, policy: [root.accountId] },
    );
    await setStakingId(root, dao, staking);

    const policy =
    {
        roles: [
            {
                name: "all",
                kind: { "Group": [alice.accountId, bob.accountId] }, // fails with kind: "Everyone" need to investigate
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

    let proposalId: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: { 'ChangePolicy': { policy } },
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );

    // Setting up a new config
    const new_config = {
        name: "new dao wohoo",
        purpose: "testing",
        metadata: '',
    };
    await registerAndDelegate(dao, staking, alice, new BN('1'));
    await registerAndDelegate(dao, staking, bob, new BN('4'));
    proposalId = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: {
                    'ChangeConfig': {
                        config: new_config,
                    }
                },
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await alice.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    await bob.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    test.deepEqual(await dao.view('get_config'),
        new_config);
});


workspaceWithoutInit.test('Policy self-lock', async (test, { alice, root, dao }) => {
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const policy =
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
    // 'staking_id' is not set, we can't delegate, so this contract got locked
    await root.call(
        dao,
        'new',
        { config, policy },
    );
    const proposalId = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: {
                    'ChangePolicy': {
                        policy,
                    }
                },
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await alice.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    let { status } = await dao.view('get_proposal', { id: proposalId });
    test.is(status, 'InProgress');
})

// proposals tests
// ---------------------------------------------------------------------------------------------------
workspace.test('basic', async (test, { alice, root, dao }) => {
    test.true(await alice.exists())
    test.true(await root.exists())
    test.true(await dao.exists())
    test.log(await dao.view('get_config'))
})

workspace.test('add_proposal fails in case of insufficient deposit', async (test, { alice, root, dao }) => {
    test.is(await dao.view('get_last_proposal_id'), 0);
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    //Try adding a proposal with 0.999... near
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

    //Checks that the same proposal doesn't fail 
    //if the deposit is at least 1 near
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

    let new_proposal: any = await dao.view('get_proposal', { id: 0 })

    test.log(new_proposal);
    test.is(new_proposal.description, 'rename the dao');
    test.is(new_proposal.proposer, 'alice.test.near')
    test.is(new_proposal.status, 'InProgress')

    test.truthy(new_proposal.kind.ChangeConfig)
    test.is(new_proposal.kind.ChangeConfig.config.name, 'sputnikdao')
    //same config as we did not execute that proposal
    test.deepEqual(await dao.view('get_config'), { name: 'sputnik', purpose: 'testing', metadata: '' })
});

workspace.test('Bob can not add proposals', async (test, { alice, root, dao }) => {
    const bob = await root.createAccount('bob');

    //First we change a policy so that Bob can't add proposals
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const newPolicy =
    {
        roles: [
            {
                name: "all",
                kind: {
                    "Group":
                        [
                            root.accountId,
                            alice.accountId
                        ]
                },
                permissions: [
                    "*:VoteApprove",
                    "*:AddProposal"
                ],
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
    let id: number = await bob.call(dao, 'add_proposal', {
        proposal: {
            description: 'change to a new policy, so that bob can not add a proposal',
            kind: {
                ChangePolicy: {
                    policy: newPolicy
                }
            }
        },
    },
        { attachedDeposit: toYocto('1') }
    )
    await voteApprove(root, dao, id);

    //Chrck that only those with a permission can add the proposal
    let errorString = await captureError(async () =>
        await bob.call(dao, 'add_proposal', {
            proposal: {
                description: 'change to a new policy',
                kind: {
                    ChangePolicy: {
                        policy: newPolicy
                    }
                }
            },
        },
            { attachedDeposit: toYocto('1') }
        )
    );
    test.regex(errorString, /ERR_PERMISSION_DENIED/);
});

workspace.test('Proposal ChangePolicy', async (test, { alice, root, dao }) => {
    test.deepEqual(await dao.view('get_proposals', { from_index: 0, limit: 10 }), []);

    //Check that we can't change policy to a policy unless it's VersionedPolicy::Current
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

    //Check that we can change to a correct policy
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const correctPolicy =
    {
        roles: [
            {
                name: "all",
                kind: {
                    "Group":
                        [
                            root.accountId,
                            alice.accountId
                        ]
                },
                permissions: [
                    "*:VoteApprove",
                    "*:AddProposal"
                ],
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
    let id: number = await alice.call(dao, 'add_proposal', {
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

    //Number of proposals = 1
    test.is(await dao.view('get_last_proposal_id'), 1);
    //Check that the proposal is added to the list of proposals
    let proposals = await dao.view('get_proposals', { from_index: 0, limit: 10 });
    let realProposal = {
        id: 0,
        proposer: alice.accountId,
        description: 'change to a new correct policy',
        kind: { ChangePolicy: { policy: correctPolicy } },
        status: 'InProgress',
        vote_counts: {},
        votes: {},
    };
    test.is(proposals[0].id, realProposal.id);
    test.is(proposals[0].proposer, realProposal.proposer);
    test.is(proposals[0].description, realProposal.description);
    test.is(proposals[0].status, realProposal.status);
    test.deepEqual(proposals[0].vote_counts, realProposal.vote_counts);
    test.deepEqual(proposals[0].votes, realProposal.votes);
    test.deepEqual(proposals[0].kind, realProposal.kind);

    //After voting on the proposal it is Approved
    await voteApprove(root, dao, id);

    test.deepEqual((await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0].vote_counts, { council: [1, 0, 0] });
    test.is((await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0].status, 'Approved');

    //Check that the policy is changed
    test.deepEqual(await dao.view('get_policy'), correctPolicy);
});

workspace.test('Proposal Transfer', async (test, { alice, root, dao }) => {
    let errorString = await captureError(async () =>
        await root.call(
            dao,
            'add_proposal', {
            proposal: {
                description: 'can not use transfer without wrong token_id and msg',
                kind: {
                    Transfer: {
                        token_id: "",
                        receiver_id: alice.accountId,
                        amount: toYocto('1'),
                        msg: "some msg"
                    }
                }
            },
        },
            {
                attachedDeposit: toYocto('1')
            })
    );
    test.regex(errorString, /ERR_BASE_TOKEN_NO_MSG/);

    const transferId: number = await root.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'transfer 1 yocto',
            kind: {
                Transfer: {
                    token_id: "",
                    receiver_id: alice,
                    amount: toYocto('1'),
                }
            }
        },
    }, { attachedDeposit: toYocto('1') })
    const initBalance: NEAR = (await alice.balance()).total;
    await voteApprove(root, dao, transferId);
    const balance: NEAR = (await alice.balance()).total;
    test.deepEqual(balance, initBalance.add(ONE_NEAR));
});

workspace.test('Proposal SetStakingContract', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    await setStakingId(root, dao, staking);

    test.is(await dao.view('get_staking_contract'), staking.accountId);

    let errorString = await captureError(async () =>
        await setStakingId(root, dao, staking)
    );
    test.regex(errorString, /ERR_STAKING_CONTRACT_CANT_CHANGE/);
});

workspace.test('Voting is only allowed for councils', async (test, { alice, root, dao }) => {
    const config = {
        name: 'sputnikdao',
        purpose: 'testing',
        metadata: ''
    }
    //add_proposal returns new proposal id
    const id: number = await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    }, { attachedDeposit: toYocto('1') })

    //Check that voting is not allowed for non councils
    //Here alice tries to vote for her proposal but she is not a council and has no permission to vote.
    const err = await captureError(async () =>
        await voteApprove(alice, dao, id)
    );
    test.log(err)
    test.true(err.includes('ERR_PERMISSION_DENIED'))

    let proposal: any = await dao.view('get_proposal', { id });
    test.log(proposal);
    test.is(proposal.status, 'InProgress');

    //Check that voting is allowed for councils
    //council (root) votes on alice's promise
    const res = await voteApprove(root, dao, id);
    proposal = await dao.view('get_proposal', { id });
    test.log(res)
    test.log(proposal);
    test.is(proposal.status, 'Approved')

    // proposal approved so now the config is equal to what alice did propose
    test.deepEqual(await dao.view('get_config'), config)
});

// If the number of votes in the group has changed (new members has been added)
//  the proposal can lose it's approved state.
//  In this case new proposal needs to be made, this one should expire
workspace.test('Proposal group changed during voting', async (test, { alice, root, dao }) => {
    const transferId: number = await root.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'give me tokens',
            kind: {
                Transfer: {
                    token_id: "",
                    receiver_id: alice,
                    amount: toYocto('1'),
                }
            }
        },
    }, { attachedDeposit: toYocto('1') })

    const addMemberToRoleId: number = await root.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'add alice',
            kind: {
                AddMemberToRole: {
                    member_id: alice,
                    role: 'council',
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });
    await voteApprove(root, dao, addMemberToRoleId);
    await voteApprove(root, dao, transferId);
    const { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'InProgress');
});

workspaceWithoutInit.test('Proposal action types', async (test, { alice, root, dao }) => {
    const user1 = await root.createAccount('user1');
    const user2 = await root.createAccount('user2');
    const user3 = await root.createAccount('user3');
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const policy =
    {
        roles: [
            {
                name: "council",
                kind: { "Group": [alice.accountId, user1.accountId, user2.accountId, user3.accountId] },
                permissions: ["*:*"],
                vote_policy: {}
            }
        ],
        default_vote_policy:
        {
            weight_kind: "RoleWeight",
            quorum: new BN('0').toString(),
            threshold: [1, 2],
        },
        proposal_bond: toYocto('1'),
        proposal_period: period,
        bounty_bond: toYocto('1'),
        bounty_forgiveness_period: period,
    };

    let config = { name: 'sputnik', purpose: 'testing', metadata: '' };

    await root.call(
        dao,
        'new',
        { config, policy },
    );

    let proposalId = await alice.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });

    // Remove proposal works
    await alice.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'RemoveProposal'
        }
    );
    let err = await captureError(async () =>
        dao.view('get_proposal', { id: proposalId })
    );
    test.regex(err, /ERR_NO_PROPOSAL/);

    err = await captureError(async () =>
        alice.call(
            dao,
            'act_proposal',
            {
                id: proposalId,
                action: 'VoteApprove'
            }
        )
    );
    test.regex(err, /ERR_NO_PROPOSAL/);

    proposalId = await alice.call(
        dao,
        'add_proposal', {
        proposal: {
            description: 'rename the dao',
            kind: {
                ChangeConfig: {
                    config
                }
            }
        },
    }, { attachedDeposit: toYocto('1') });

    err = await captureError(async () =>
        alice.call(
            dao,
            'act_proposal',
            {
                id: proposalId,
                action: 'AddProposal'
            }
        )
    );
    test.regex(err, /ERR_WRONG_ACTION/);

    // Check if every vote counts
    await user1.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove'
        }
    );
    await user2.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteReject'
        }
    );
    await alice.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteRemove'
        }
    );
    {
        const { vote_counts, votes } = await dao.view('get_proposal', { id: proposalId });
        test.deepEqual(vote_counts.council, [1, 1, 1]);
        test.deepEqual(votes, {
            [alice.accountId]: 'Remove',
            [user1.accountId]: 'Approve',
            [user2.accountId]: 'Reject'
        });
    }

    // Finalize proposal will panic if not exired or failed
    err = await captureError(async () =>
        alice.call(
            dao,
            'act_proposal',
            {
                id: proposalId,
                action: 'Finalize'
            }
        )
    );
    test.regex(err, /ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED/);
});

workspace.test('Proposal transfer ft', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
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
    const transferId: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'transfer tokens to me',
                kind: {
                    Transfer: {
                        token_id: testToken.accountId,
                        receiver_id: alice.accountId,
                        amount: '10',
                    }
                }
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await voteApprove(root, dao, transferId);
    const { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Approved');
});

// views tests
// ------------------------------------------------------------------------
workspace.test('View method version', async (test, { alice, root, dao }) => {
    test.log('Version:');
    test.log(await dao.view('version'));
    test.is(await dao.view('version'), "2.0.0");
});

workspace.test('View method get_config', async (test, { root }) => {
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

workspace.test('View method get_policy', async (test, { root }) => {
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

workspace.test('View method get_staking_contract', async (test, { alice, root, dao }) => {
    test.is(await dao.view('get_staking_contract'), null);

    //To set the staking_id
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    await setStakingId(root, dao, staking);

    test.is(await dao.view('get_staking_contract'), staking.accountId);
});

workspace.test('View has_blob', async (test, { alice, root, dao }) => {
    const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');
    const result = await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();
    const hash = result.parseResult<String>();
    test.true(await dao.view('has_blob', { hash: hash }));
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
    test.false(await dao.view('has_blob', { hash: hash }));
});

workspace.test('View get_locked_storage_amount', async (test, { alice, root, dao }) => {
    const beforeProposal = new BN(await dao.view('get_locked_storage_amount'));
    test.log('Locked amount: ' + beforeProposal);
    await root.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'adding some bytes',
                kind: 'Vote',
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    const afterProposal = new BN(await dao.view('get_locked_storage_amount'));
    test.assert(beforeProposal.lt(afterProposal));
});

workspace.test('View get_available_amount', async (test, { alice, root, dao }) => {
    const beforeProposal = new BN(await dao.view('get_available_amount'));
    test.log('Available amount: ' + beforeProposal);
    await root.call(dao, 'add_proposal',
        {
            proposal: {
                description: 'adding some bytes',
                kind: 'Vote',
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    const afterProposal = new BN(await dao.view('get_available_amount'));
    test.assert(beforeProposal.gt(afterProposal));
});

workspace.test('View methods for delegation', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');
    const bob = await root.createAccount('bob');

    await setStakingId(root, dao, staking);

    let result = await registerAndDelegate(dao, staking, alice, randomAmount);
    result = await registerAndDelegate(dao, staking, bob, randomAmount.muln(2));

    //Test delegation_balance_of
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: alice })),
        randomAmount);
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: bob })),
        randomAmount.muln(2));

    //Test delegation_total_supply
    test.deepEqual(new BN(
        await dao.view('delegation_total_supply')),
        randomAmount.muln(3));

    //Test delegation_balance_ratio
    test.deepEqual(
        await dao.view('delegation_balance_ratio', { account_id: alice }),
        [
            await dao.view('delegation_balance_of', { account_id: alice }),
            await dao.view('delegation_total_supply')
        ]);
});

workspace.test('View methods for proposals', async (test, { alice, root, dao }) => {
    //Test get_last_proposal_id
    test.is(await dao.view('get_last_proposal_id'), 0);

    //Test get_proposals
    test.deepEqual(await dao.view('get_proposals', { from_index: 0, limit: 100 }), []);

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
        kind: { ChangeConfig: { config } },
        status: 'InProgress',
        vote_counts: {},
        votes: {}
    };

    const proposalAlice: any = await dao.view('get_proposal', { id: 0 });

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
    const proposals: any = await dao.view('get_proposals', { from_index: 0, limit: 100 });
    test.is(proposals[0].proposer, realProposalAlice.proposer);
    test.is(proposals[0].description, realProposalAlice.description);
    test.is(proposals[0].status, realProposalAlice.status);
    test.deepEqual(proposals[0].vote_counts, realProposalAlice.vote_counts);
    test.deepEqual(proposals[0].votes, realProposalAlice.votes);
    test.deepEqual(proposals[0].kind, realProposalAlice.kind);

    //Should panic if the proposal with the given id doesn't exist
    const errorString = await captureError(async () =>
        await dao.view('get_proposal', { id: 10 })
    );
    test.regex(errorString, /ERR_NO_PROPOSAL/);
});

workspace.test('View methods for bounties', async (test, { alice, root, dao }) => {
    //Test get_last_bounty_id
    test.is(await dao.view('get_last_bounty_id'), 0);
    //Test get_bounties
    test.deepEqual(await dao.view('get_bounties', { from_index: 0, limit: 100 }), []);

    const testToken = await initTestToken(root);
    const proposalId = await proposeBounty(alice, dao, testToken);
    const bounty = {
        id: 0,
        description: 'test_bounties',
        token: testToken.accountId,
        amount: '19000000000000000000000000',
        times: 3,
        max_deadline: DEADLINE
    }
    await voteOnBounty(root, dao, proposalId);

    //Test get_last_bounty_id
    test.is(await dao.view('get_last_bounty_id'), 1);
    //Test get_bounties
    test.deepEqual(await dao.view('get_bounties', { from_index: 0, limit: 100 }), [bounty]);
    //Test get_bounty
    test.deepEqual(await dao.view('get_bounty', { id: 0 }), bounty);

    await claimBounty(alice, dao, proposalId);

    //Test get_bounty_number_of_claims
    test.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 1);
    //Test get_bounty_claims
    const realClaim = {
        bounty_id: 0,
        deadline: DEADLINE,
        completed: false
    };
    const claims: any = await dao.view('get_bounty_claims', { account_id: alice.accountId });
    test.is(claims[0].bounty_id, realClaim.bounty_id);
    test.is(claims[0].deadline, realClaim.deadline);
    test.is(claims[0].completed, realClaim.completed);

    //Should panic if the bounty with the given id doesn't exist
    const errorString = await captureError(async () =>
        await dao.view('get_bounty', { id: 10 })
    );
    test.regex(errorString, /ERR_NO_BOUNTY/);
});
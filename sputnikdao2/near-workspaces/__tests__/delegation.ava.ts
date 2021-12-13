import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';

const regCost = STORAGE_PER_BYTE.mul(new BN(16));

async function setStakingId(root: NearAccount, dao: NearAccount, staking: NearAccount) {
    // Setting staking id
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal: { description: 'test', kind: { "SetStakingContract": { "staking_id": staking.accountId } } },
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

async function registerAndDelegate(dao: NearAccount, staking: NearAccount, account: NearAccount, amount: BN) {

    await staking.call(dao, 'register_delegation', { account_id: account },
        { attachedDeposit: regCost }
    );
    await staking.call(
        dao,
        'delegate',
        {
            account_id: account,
            amount: amount.toString(),
        }
    )
}

workspace.test('Register delegation', async (test, { root, dao, alice }) => {
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


    await registerAndDelegate(dao, staking, alice, new BN(1));

    // Check that delegation appears in `delegations` LookupMap.
    const bal: BN = new BN(await dao.view('delegation_balance_of', { account_id: alice }));
    const total: BN = new BN(await dao.view('delegation_total_supply'));
    test.deepEqual(bal, new BN(1));
    test.deepEqual(total, new BN(1));

});


workspace.test('Delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');
    const bob = await root.createAccount('bob');

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(dao, 'delegate',
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
        root.call(dao, 'delegate',
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

    await registerAndDelegate(dao, staking, alice, randomAmount);
    await registerAndDelegate(dao, staking, bob, randomAmount.muln(2));
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: alice })
        ),randomAmount);
    test.deepEqual(new BN(
        await dao.view('delegation_balance_of', { account_id: bob })
        ),randomAmount.muln(2));
    test.deepEqual(new BN(
        await dao.view('delegation_total_supply')
        ),randomAmount.muln(3));
});


/*
workspace.test('Bounty claim from bash', async (test, { alice, root, dao }) => {
    const deadline = new BN('1925376849430593581')
    const bounty = {
        description: 'do the thing',
        amount: toYocto('19'),
        times: 3,
        max_deadline: deadline
    }
    let proposalId = await root.call(dao, 'add_proposal', {
        proposal: {
            description: 'test bounty',
            kind: {
                AddBounty: {
                    bounty
                }
            }
        },
    },
        { attachedDeposit: toYocto('1') });

    const result = await root.call_raw(dao, 'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        });
    test.assert(result.logsContain('council'));

    await root.call(dao, 'bounty_claim',
        {
            id: proposalId,
            deadline: deadline,
        }, { attachedDeposit: toYocto('1') });
    let bountyClaims = await dao.view('get_bounty_claims', { account_id: root });
    let { start_time, ...bountyClaim } = bountyClaims[0];
    test.deepEqual(bountyClaim,
        {
            bounty_id: 0,
            completed: false,
            deadline: '1925376849430593581',
        });
});
*/
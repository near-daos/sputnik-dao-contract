import {
    BN,
    NearAccount,
    captureError,
    toYocto,
    tGas,
} from 'near-workspaces';
import {
    initStaking,
    initTestToken,
    STORAGE_PER_BYTE,
    setStakingId,
    registerAndDelegate,
    regCost,
    initWorkspace,
} from './utils';

const test = initWorkspace();

test('Register delegation', async (t) => {
    const { root, dao, alice } = t.context.accounts;
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);

    // set staking
    await setStakingId(root, dao, staking);

    await registerAndDelegate(dao, staking, alice, new BN(1));

    // Check that delegation appears in `delegations` LookupMap.
    let bal: BN = new BN(
        await dao.view('delegation_balance_of', { account_id: alice }),
    );
    t.deepEqual(bal, new BN(1));
    const total: BN = new BN(await dao.view('delegation_total_supply'));
    t.deepEqual(total, new BN(1));
});

test(
    'Register delegation fail',
    async (t) => {
        const { root, dao, alice } = t.context.accounts;
        const testToken = await initTestToken(root);
        const staking = await initStaking(root, dao, testToken);

        // Staking id not set
        let errorString = await captureError(async () =>
            staking.call(
                dao,
                'register_delegation',
                { account_id: alice },
                { attachedDeposit: regCost },
            ),
        );
        t.regex(errorString, /ERR_NO_STAKING/);

        await setStakingId(root, dao, staking);
        // Can only be called by the `staking_id`
        errorString = await captureError(async () =>
            root.call(
                dao,
                'register_delegation',
                { account_id: alice },
                { attachedDeposit: regCost },
            ),
        );
        t.regex(errorString, /ERR_INVALID_CALLER/);

        // Attached deposit is handled correctly
        await captureError(async () =>
            root.call(
                dao,
                'register_delegation',
                { account_id: alice },
                { attachedDeposit: regCost.add(new BN(1)) },
            ),
        );
        await captureError(async () =>
            root.call(
                dao,
                'register_delegation',
                { account_id: alice },
                { attachedDeposit: regCost.sub(new BN(1)) },
            ),
        );
    },
);

test('Delegation', async (t) => {
    const { root, dao, alice } = t.context.accounts;
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');
    const bob = await root.createSubAccount('bob');

    // set staking
    await setStakingId(root, dao, staking);

    let result = await registerAndDelegate(dao, staking, alice, randomAmount);
    t.deepEqual(
        [new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [new BN('0'), randomAmount, randomAmount],
    );
    result = await registerAndDelegate(dao, staking, bob, randomAmount.muln(2));
    t.deepEqual(
        [new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [new BN('0'), randomAmount.muln(2), randomAmount.muln(3)],
    );
    t.deepEqual(
        new BN(await dao.view('delegation_balance_of', { account_id: alice })),
        randomAmount,
    );
    t.deepEqual(
        new BN(await dao.view('delegation_balance_of', { account_id: bob })),
        randomAmount.muln(2),
    );
    t.deepEqual(
        new BN(await dao.view('delegation_total_supply')),
        randomAmount.muln(3),
    );
});

test('Delegation fail', async (t) => {
    const { root, dao, alice } = t.context.accounts;
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('10087687667869');

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(dao, 'delegate', {
            account_id: alice,
            amount: randomAmount,
        }),
    );
    t.regex(errorString, /ERR_NO_STAKING/);

    // set staking
    await setStakingId(root, dao, staking);

    // Check that it can only be called by the `staking_id`
    errorString = await captureError(async () =>
        root.call(dao, 'delegate', {
            account_id: alice,
            amount: randomAmount,
        }),
    );
    t.regex(errorString, /ERR_INVALID_CALLER/);

    // Can't be called without previos registration
    errorString = await captureError(async () =>
        staking.call(dao, 'delegate', {
            account_id: 'not-registered-account.bob',
            amount: randomAmount,
        }),
    );
    t.regex(errorString, /ERR_NOT_REGISTERED/);
});

test('Undelegate', async (t) => {
    const { root, dao, alice } = t.context.accounts;
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('44887687667868');

    // set staking
    await setStakingId(root, dao, staking);

    await registerAndDelegate(dao, staking, alice, randomAmount);

    // Check that amount is subtracted correctly
    const result: string[3] = await staking.call(dao, 'undelegate', {
        account_id: alice,
        amount: randomAmount.divn(2).toString(),
    });
    t.deepEqual(
        [new BN(result[0]), new BN(result[1]), new BN(result[2])],
        [randomAmount, randomAmount.divn(2), randomAmount.divn(2)],
    );
});

test('Undelegate fail', async (t) => {
    const { root, dao, alice } = t.context.accounts;
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const randomAmount = new BN('44887687667868');

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(dao, 'undelegate', {
            account_id: alice,
            amount: randomAmount,
        }),
    );
    t.regex(errorString, /ERR_NO_STAKING/);

    // Set staking
    await setStakingId(root, dao, staking);

    // Check that it can only be called by the `staking_id`
    errorString = await captureError(async () =>
        root.call(dao, 'undelegate', {
            account_id: alice,
            amount: randomAmount,
        }),
    );
    t.regex(errorString, /ERR_INVALID_CALLER/);

    await registerAndDelegate(dao, staking, alice, randomAmount);
    // Check that a user can't remove more than it delegated
    errorString = await captureError(async () =>
        staking.call(dao, 'undelegate', {
            account_id: alice,
            amount: randomAmount.addn(1).toString(),
        }),
    );
    t.regex(errorString, /ERR_INVALID_STAKING_CONTRACT/);
});

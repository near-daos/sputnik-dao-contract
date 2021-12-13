import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';

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

workspace.test('Register delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const regCost = STORAGE_PER_BYTE.mul(new BN(16));

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

    await staking.call(dao, 'register_delegation', { account_id: alice },
        { attachedDeposit: regCost }
    );
    await staking.call(
        dao,
        'delegate',
        {
            account_id: alice,
            amount: new BN(1),
        }
    )

    // Check that delegation appears in `delegations` LookupMap.
    const bal: BN = new BN(await dao.view('delegation_balance_of', { account_id: alice }));
    const total: BN = new BN(await dao.view('delegation_total_supply'));
    test.deepEqual(bal, new BN(1));
    test.deepEqual(total, new BN(1));

});


workspace.test('Delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const regCost = STORAGE_PER_BYTE.mul(new BN(16));

    // Should panic if `staking_id` is `None`
    let errorString = await captureError(async () =>
        staking.call(dao, 'delegate',
            {
                account_id: alice,
                amount: new BN(1),
            })
    );
    test.regex(errorString, /ERR_NO_STAKING/);

    await setStakingId(root, dao, staking)
});
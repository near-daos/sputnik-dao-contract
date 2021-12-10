import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';

workspace.test('Register delegation', async (test, { root, dao, alice }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    const regCost = STORAGE_PER_BYTE.mul(new BN(16));
    let errorString = await captureError(async () =>
        staking.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost }));
    
    test.regex(errorString, /ERR_NO_STAKING/); // Staking id not set 
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

    errorString = await captureError(async () =>
        root.call(dao, 'register_delegation', { account_id: alice },
            { attachedDeposit: regCost }));
    test.regex(errorString, /ERR_INVALID_CALLER/); // only stake account can call register_delegation

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

    const bal: BN = new BN(await dao.view('delegation_balance_of', { account_id: alice }));
    const total: BN = new BN(await dao.view('delegation_total_supply'));
    test.deepEqual(bal, new BN(1));
    test.deepEqual(total, new BN(1));

});
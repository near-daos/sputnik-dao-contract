import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken } from './utils';

workspace.test('Register delegation', async (test, {root, dao }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal: {description: 'test', kind: {"SetStakingContract": {"staking_id": staking.accountId}}},
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

});
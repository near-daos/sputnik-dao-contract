import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken } from './utils';

workspace.test('Register delegation check', async (test, {root, dao }) => {
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
});
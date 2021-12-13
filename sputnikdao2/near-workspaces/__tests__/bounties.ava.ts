import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken } from './utils';

workspace.test('Bounty claim', async (test, {alice, root, dao }) => {
    const bounty = {
        description: 'test_bounties',
        token: alice,
        amount: '1',
        times: 2,
        max_deadline: '1000'
    }
    await alice.call(dao, 'add_proposal', {
        proposal: {
            description: 'add_new_bounty',
            kind: {
                AddBounty: {
                    bounty
                }
            }
        },
    },
        { attachedDeposit: toYocto('1') })
});
import { Workspace, NearAccount } from 'near-workspaces-ava';
import { workspace } from './utils';

workspace.test('basic', async (test, {alice, root, dao})=>{
test.true(await alice.exists())
test.true(await root.exists())
test.true(await dao.exists())
})
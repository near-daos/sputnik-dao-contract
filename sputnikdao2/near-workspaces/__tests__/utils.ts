import { Workspace, NearAccount } from 'near-workspaces-ava';

async function initWorkspace(root: NearAccount) {
    const workspace = Workspace.init();

    const alice = await root.createAccount('alice');
    console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N

    const config = { name: 'sputnik', purpose: 'testing', metadata: '' }
    const policy = []

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            method: 'new',
            args: { config, policy }
        }
    );

    console.log('dao\'s balance is: ' + (await dao.balance()).total) //~100N

    return { alice, dao };
}

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root)
});
import { Workspace, NearAccount } from 'near-workspaces-ava';

async function initWorkspace(root: NearAccount) {
    const workspace = Workspace.init();
    
    const alice = await root.createAccount('alice');

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
    );

    return { alice, dao };
}

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root)
});
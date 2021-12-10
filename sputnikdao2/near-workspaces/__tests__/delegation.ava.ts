import { Workspace, BN, NearAccount, captureError, toYocto, tGas } from 'near-workspaces-ava';


export const workspace = Workspace.init(async ({ root }) => {
    const alice = await root.createAccount('alice');


    const config = {name: 'test', purpose: 'testing', metadata: ""};
    
    // Create a subaccount of the root account, and also deploy a contract to it
    const dao = await root.createAndDeploy(
      'dao',
      '../res/sputnikdao2.wasm',
      {
          method: 'new',
          args: {config, policy: []},
      }
    );
  
    return { alice, dao };
  });
  

workspace.test('Title', async (test, { dao }) => {

});
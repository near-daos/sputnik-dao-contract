import { Workspace, NearAccount, BN, toYocto } from 'near-workspaces-ava';

async function initWorkspace(root: NearAccount) {
    const workspace = Workspace.init();

    const alice = await root.createAccount('alice');
    console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N
    const errorString = "";
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' }
    const policy = [root.accountId]

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            method: 'new',
            args: { config, policy },
            initialBalance: toYocto('200'),
        }
    );

    console.log('dao\'s balance is: ' + (await dao.balance()).total) //~100N

    return { alice, dao };
}

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root)
});

export async function initTestToken(root: NearAccount) {
    const testToken = await root.createAndDeploy(
        'test-token',
        '../../test-token/res/test_token.wasm',
        {
            method: 'new',
            initialBalance: toYocto('200'),
        }
    );
    return testToken;
}

export async function initStaking(root:NearAccount, dao: NearAccount, testToken: NearAccount) {
    const staking = await root.createAndDeploy(
        'staking',
        '../../sputnik-staking/res/sputnik_staking.wasm',
        {
            method: 'new',
            args: {owner_id: dao, token_id: testToken, unstake_period: '100000000000'},
            initialBalance: toYocto('100'),
        }
    );
    return staking
}
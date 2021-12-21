import { Workspace, NearAccount, BN, toYocto } from 'near-workspaces-ava';


async function initWorkspace(root: NearAccount) {

    const alice = await root.createAccount('alice');
    // console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N

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

    // console.log('dao\'s balance is: ' + (await dao.balance()).total) //~200N

    return { alice, dao };
}

export const STORAGE_PER_BYTE = new BN('10000000000000000000');

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root)
});

export const workspaceWithoutInit = Workspace.init(async ({ root }) => {
    const alice = await root.createAccount('alice');

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            initialBalance: toYocto('200'),
        }
    );
    return { alice, dao };
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

export async function initStaking(root: NearAccount, dao: NearAccount, testToken: NearAccount) {
    const staking = await root.createAndDeploy(
        'staking',
        '../../sputnik-staking/res/sputnik_staking.wasm',
        {
            method: 'new',
            args: { owner_id: dao, token_id: testToken, unstake_period: '100000000000' },
            initialBalance: toYocto('100'),
        }
    );
    return staking
}

export async function setStakingId(root: NearAccount, dao: NearAccount, staking: NearAccount) {
    // Setting staking id
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal:
            {
                description: 'test',
                kind: { "SetStakingContract": { "staking_id": staking.accountId } }
            },
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

export const regCost = STORAGE_PER_BYTE.mul(new BN(16));

export async function registerAndDelegate(dao: NearAccount, staking: NearAccount, account: NearAccount, amount: BN) {
    await staking.call(dao, 'register_delegation', { account_id: account },
        { attachedDeposit: regCost }
    );
    const res: string[3] = await staking.call(
        dao,
        'delegate',
        {
            account_id: account,
            amount: amount.toString(),
        }
    );
    return res;
}

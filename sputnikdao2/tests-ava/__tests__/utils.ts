import { Workspace, NearAccount, BN, toYocto, tGas } from 'near-workspaces-ava';

async function initWorkspace(root: NearAccount) {
    const alice = await root.createAccount('alice');
    // console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N

    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
    const policy = [root.accountId];

    //for short let's call it just dao
    const dao = await root.createAndDeploy('dao', '../res/sputnikdao2.wasm', {
        method: 'new',
        args: { config, policy },
        initialBalance: toYocto('200'),
    });

    // console.log('dao\'s balance is: ' + (await dao.balance()).total) //~200N

    return { alice, dao };
}

export const STORAGE_PER_BYTE = new BN('10000000000000000000');

export const workspace = Workspace.init(async ({ root }) => {
    return initWorkspace(root);
});

export const workspaceWithoutInit = Workspace.init(async ({ root }) => {
    const alice = await root.createAccount('alice');

    //for short let's call it just dao
    const dao = await root.createAndDeploy('dao', '../res/sputnikdao2.wasm', {
        initialBalance: toYocto('200'),
    });
    return { alice, dao };
});

export const workspaceWithFactory = Workspace.init(async ({ root }) => {
    const factory = await root.createAndDeploy(
        'factory',
        '../../sputnikdao-factory2/res/sputnikdao_factory2.wasm',
        {
            initialBalance: toYocto('500'),
        },
    );
    await factory.call(factory.accountId, 'new', {}, { gas: tGas(300) });
    return { factory };
});

export async function initTestToken(root: NearAccount) {
    const testToken = await root.createAndDeploy(
        'test-token',
        '../../test-token/res/test_token.wasm',
        {
            method: 'new',
            initialBalance: toYocto('200'),
        },
    );
    return testToken;
}

export async function initStaking(
    root: NearAccount,
    dao: NearAccount,
    testToken: NearAccount,
) {
    const staking = await root.createAndDeploy(
        'staking',
        '../../sputnik-staking/res/sputnik_staking.wasm',
        {
            method: 'new',
            args: {
                owner_id: dao,
                token_id: testToken,
                unstake_period: '100000000000',
            },
            initialBalance: toYocto('100'),
        },
    );
    return staking;
}

export async function setStakingId(
    root: NearAccount,
    dao: NearAccount,
    staking: NearAccount,
) {
    // Setting staking id
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: { SetStakingContract: { staking_id: staking.accountId } },
            },
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
    await root.call(dao, 'act_proposal', {
        id: proposalId,
        action: 'VoteApprove',
    });
}

export const regCost = STORAGE_PER_BYTE.mul(new BN(16));

export async function registerAndDelegate(
    dao: NearAccount,
    staking: NearAccount,
    account: NearAccount,
    amount: BN,
) {
    await staking.call(
        dao,
        'register_delegation',
        { account_id: account },
        { attachedDeposit: regCost },
    );
    const res: string[3] = await staking.call(dao, 'delegate', {
        account_id: account,
        amount: amount.toString(),
    });
    return res;
}

export const DEADLINE = '1925376849430593581';
export const BOND = toYocto('1');

export async function proposeBounty(
    alice: NearAccount,
    dao: NearAccount,
    token: NearAccount,
) {
    const bounty = {
        description: 'test_bounties',
        token: token.accountId,
        amount: '19000000000000000000000000',
        times: 3,
        max_deadline: DEADLINE,
    };
    const proposalId: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'add_new_bounty',
                kind: {
                    AddBounty: {
                        bounty,
                    },
                },
            },
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
    return proposalId;
}

export async function proposeBountyWithNear(
    alice: NearAccount,
    dao: NearAccount,
) {
    const bounty = {
        description: 'test_bounties_with_near_token',
        token: '',
        amount: '19000000000000000000000000',
        times: 3,
        max_deadline: DEADLINE,
    };
    const proposalId: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'add_new_bounty',
                kind: {
                    AddBounty: {
                        bounty,
                    },
                },
            },
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
    return proposalId;
}

export async function voteOnBounty(
    root: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(50),
        },
    );
}

export async function claimBounty(
    alice: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    await alice.call(
        dao,
        'bounty_claim',
        {
            id: proposalId,
            deadline: DEADLINE,
        },
        {
            attachedDeposit: BOND,
        },
    );
}

export async function doneBounty(
    alice: NearAccount,
    bob: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    await alice.call(
        dao,
        'bounty_done',
        {
            id: proposalId,
            account_id: bob,
            description: 'This bounty is done',
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
}

export async function giveupBounty(
    alice: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    return await alice.call(dao, 'bounty_giveup', { id: proposalId });
}

export async function giveupBountyRaw(
    alice: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    return await alice.call_raw(dao, 'bounty_giveup', { id: proposalId });
}

export async function voteApprove(
    root: NearAccount,
    dao: NearAccount,
    proposalId: number,
) {
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(100),
        },
    );
}

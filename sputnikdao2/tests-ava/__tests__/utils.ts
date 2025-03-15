import { Worker, NearAccount, BN, toYocto, tGas, KeyPair } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';
import * as fs from 'fs';

export async function deployAndInit({
    root,
    subContractId,
    code,
    init,
    initialBalance,
}: {
    root: NearAccount;
    subContractId: string;
    code: Uint8Array | string;
    init?: {
        methodName: string;
        args?: Record<string, unknown>;
        options?: {
            gas?: string | BN;
            attachedDeposit?: string | BN;
            signWithKey?: KeyPair;
        }
    };
    initialBalance?: string;
}): Promise<NearAccount> {
    const contract = await root.createSubAccount(subContractId, {
        initialBalance,
    });
    const result = await contract.deploy(code);
    if (result.failed) {
        throw result.Failure;
    }
    if (init) {
        await contract.call(contract, init.methodName, init.args ?? {}, init.options);
    }
    return contract;
}

export function initWorkspace(options?: { skipInit?: boolean, factory?: boolean}) {
    const test = anyTest as TestFn<{
        worker: Worker;
        accounts: Record<string, NearAccount>;
    }>;

    test.beforeEach(async (t) => {
        // Init the worker and start a Sandbox server
        const worker = await Worker.init();

        // Create accounts
        const root = worker.rootAccount;
        const alice = await root.createSubAccount('alice');
        // console.log('alice\'s balance is: ' + (await alice.balance()).total) //100N

        const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
        const policy = [root.accountId];

        //for short let's call it just dao
        const dao = await deployAndInit({
            root,
            subContractId: 'dao',
            code: '../res/sputnikdao2.wasm',
            init: options?.skipInit ? undefined : {
                methodName: 'new',
                args: { config, policy },
            },
            initialBalance: toYocto('200'),
        });

        const factory = options?.factory ? await deployAndInit({
            root,
            subContractId: 'factory',
            code: '../../sputnikdao-factory2/res/sputnikdao_factory2.wasm',
            init: {
                methodName: 'new',
                args: {},
                options: {
                    gas: tGas(300),
                },
            }, // 300 Tags
            initialBalance: toYocto('500'),
        }) : undefined;

        // Save state for test runs, it is unique for each test
        t.context.worker = worker;
        t.context.accounts = {
            root,
            alice,
            dao,
            factory,
        };
    });

    test.afterEach.always(async (t) => {
        // Stop Sandbox server
        await t.context.worker.tearDown().catch((error) => {
            console.log('Failed to stop the Sandbox:', error);
        });
    });

    return test;
}

export const STORAGE_PER_BYTE = new BN('10000000000000000000');

export async function initTestToken(root: NearAccount) {
    const testToken = await deployAndInit({
        root,
        subContractId: 'test-token',
        code: '../../test-token/res/test_token.wasm',
        init: {
            methodName: 'new',
        },
        initialBalance: toYocto('200'),
    });
    return testToken;
}

export async function initStaking(
    root: NearAccount,
    dao: NearAccount,
    testToken: NearAccount,
) {
    const staking = await deployAndInit({
        root,
        subContractId: 'staking',
        code: '../../sputnik-staking/res/sputnik_staking.wasm',
        init: {
            methodName: 'new',
            args: {
                owner_id: dao,
                token_id: testToken,
                unstake_period: '100000000000',
            },
        },
        initialBalance: toYocto('100'),
    });
    return staking;
}

export async function setStakingId(
    root: NearAccount,
    dao: NearAccount,
    staking: NearAccount,
) {
    // Setting staking id
    const proposalId: number = await root.call(
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
        proposal: await getProposalKind(dao, proposalId),
    });
}

export const regCost = STORAGE_PER_BYTE.mul(new BN(16));
export const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');

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
            proposal: await getProposalKind(dao, proposalId),
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
    return await alice.callRaw(dao, 'bounty_giveup', { id: proposalId });
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
            proposal: await getProposalKind(dao, proposalId),
        },
        {
            gas: tGas(100),
        },
    );
}

export async function getProposalKind(dao: NearAccount, proposalId: number) {
    const propolsal: any = await dao.view("get_proposal", { id: proposalId });
    return propolsal.kind;
}

export type ProposalStatus = 'InProgress' | 'Approved' | 'Rejected' | 'Removed' | 'Expired' | 'Moved' | 'Failed';

export interface Proposal {
    status: ProposalStatus
};

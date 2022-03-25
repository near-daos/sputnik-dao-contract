import {
    BN,
    NearAccount,
    captureError,
    toYocto,
    tGas,
    DEFAULT_FUNCTION_CALL_GAS,
    Gas,
    NEAR,
} from 'near-workspaces-ava';
import {
    workspace,
    initStaking,
    initTestToken,
    STORAGE_PER_BYTE,
    workspaceWithoutInit,
    workspaceWithFactory,
} from './utils';
import { voteApprove } from './utils';
import {
    DEADLINE,
    BOND,
    proposeBounty,
    proposeBountyWithNear,
    voteOnBounty,
    claimBounty,
    doneBounty,
} from './utils';
import * as fs from 'fs';

const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');

workspaceWithFactory.test(
    'Upgrade self using factory',
    async (test, { root, factory }) => {
        const config = {
            name: 'testdao',
            purpose: 'to test',
            metadata: '',
        };
        const policy = [root.accountId];
        const params = {
            config,
            policy,
        };

        await root.call(
            factory,
            'create',
            {
                name: 'testdao',
                args: Buffer.from(JSON.stringify(params)).toString('base64'),
            },
            {
                attachedDeposit: toYocto('10'),
                gas: tGas(300),
            },
        );

        test.deepEqual(await factory.view('get_dao_list', {}), [
            'testdao.factory.test.near',
        ]);
        const hash = await factory.view('get_default_code_hash', {});

        const proposalId: number = await root.call(
            'testdao.factory.test.near',
            'add_proposal',
            {
                proposal: {
                    description: 'proposal to test',
                    kind: {
                        UpgradeSelf: {
                            hash: hash,
                        },
                    },
                },
            },
            {
                attachedDeposit: toYocto('1'),
            },
        );
        test.is(proposalId, 0);

        await root.call(
            'testdao.factory.test.near',
            'act_proposal',
            {
                id: proposalId,
                action: 'VoteApprove',
            },
            {
                gas: tGas(300),
            },
        );
    },
);

workspaceWithoutInit.test(
    'Upgrade self negative',
    async (test, { root, dao }) => {
        const config = { name: 'sputnik', purpose: 'testing', metadata: '' };

        // NOT INITIALIZED
        let err = await captureError(async () =>
            root.call(dao, 'store_blob', DAO_WASM_BYTES, {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            }),
        );
        test.regex(err, /ERR_CONTRACT_IS_NOT_INITIALIZED/);

        // Initializing contract
        await root.call(dao, 'new', { config, policy: [root.accountId] });

        // not enough deposit
        err = await captureError(async () =>
            root.call(dao, 'store_blob', DAO_WASM_BYTES, {
                attachedDeposit: toYocto('1'),
                gas: tGas(300),
            }),
        );
        test.regex(err, /ERR_NOT_ENOUGH_DEPOSIT/);

        await root.call(dao, 'store_blob', DAO_WASM_BYTES, {
            attachedDeposit: toYocto('200'),
            gas: tGas(300),
        });

        // Already exists
        err = await captureError(async () =>
            root.call(dao, 'store_blob', DAO_WASM_BYTES, {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            }),
        );
        test.regex(err, /ERR_ALREADY_EXISTS/);
    },
);

workspace.test('Remove blob', async (test, { root, dao, alice }) => {
    const hash: String = await root.call(dao, 'store_blob', DAO_WASM_BYTES, {
        attachedDeposit: toYocto('200'),
        gas: tGas(300),
    });

    // fails if hash is wrong
    let err = await captureError(async () =>
        root.call(dao, 'remove_blob', {
            hash: 'HLBiX51txizmQzZJMrHMCq4u7iEEqNbaJppZ84yW7628', // some_random hash
        }),
    );
    test.regex(err, /ERR_NO_BLOB/);

    // Can only be called by the original storer
    err = await captureError(async () =>
        alice.call(dao, 'remove_blob', {
            hash: hash,
        }),
    );
    test.regex(err, /ERR_INVALID_CALLER/);

    // blob is removed with payback
    const rootAmountBeforeRemove = (await root.balance()).total;
    await root.call(dao, 'remove_blob', {
        hash: hash,
    });
    const rootAmountAfterRemove = (await root.balance()).total;
    test.false(await dao.view('has_blob', { hash: hash }));
    test.assert(rootAmountAfterRemove.gt(rootAmountBeforeRemove));
});

workspace.test(
    'Callback for BountyDone with NEAR token',
    async (test, { alice, root, dao }) => {
        //During the callback the number bounty_claims_count should decrease
        const proposalId = await proposeBountyWithNear(alice, dao);
        await voteOnBounty(root, dao, proposalId);
        await claimBounty(alice, dao, proposalId);
        await doneBounty(alice, alice, dao, proposalId);
        //Before the bounty is done there is 1 claim
        test.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 1);
        const balanceBefore: NEAR = (await alice.balance()).total;
        //During the callback this number is decreased
        await voteOnBounty(root, dao, proposalId + 1);
        const balanceAfter: NEAR = (await alice.balance()).total;
        test.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 0);
        test.assert(balanceBefore.lt(balanceAfter));
    },
);

workspace.test(
    'Callback for BountyDone ft token fail',
    async (test, { alice, root, dao }) => {
        //Test the callback with Failed proposal status
        const testTokenFail = await initTestToken(root);
        const proposalIdFail = await proposeBounty(alice, dao, testTokenFail);
        await dao.call(
            testTokenFail,
            'mint',
            {
                account_id: dao,
                amount: '1000000000',
            },
            {
                gas: tGas(50),
            },
        );
        await alice.call(
            testTokenFail,
            'storage_deposit',
            {
                account_id: alice.accountId,
                registration_only: true,
            },
            {
                attachedDeposit: toYocto('90'),
            },
        );
        await voteOnBounty(root, dao, proposalIdFail);
        await claimBounty(alice, dao, proposalIdFail);
        await doneBounty(alice, alice, dao, proposalIdFail);
        await voteOnBounty(root, dao, proposalIdFail + 1);
        //Proposal should be Failed
        let { status } = await dao.view('get_proposal', {
            id: proposalIdFail + 1,
        });
        test.is(status, 'Failed');
    },
);

workspace.test(
    'Callback for BountyDone ft token',
    async (test, { alice, root, dao }) => {
        //Test correct callback
        const testToken = await initTestToken(root);
        await dao.call(
            testToken,
            'mint',
            {
                account_id: dao,
                amount: '1000000000',
            },
            {
                gas: tGas(50),
            },
        );
        await alice.call(
            testToken,
            'storage_deposit',
            {
                account_id: alice.accountId,
                registration_only: true,
            },
            {
                attachedDeposit: toYocto('90'),
            },
        );
        const bounty = {
            description: 'test_bounties',
            token: testToken.accountId,
            amount: '10',
            times: 3,
            max_deadline: DEADLINE,
        };
        let proposalId: number = await alice.call(
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
        await voteOnBounty(root, dao, proposalId);
        await claimBounty(alice, dao, proposalId);
        await doneBounty(alice, alice, dao, proposalId);
        //Before the bounty is done there is 1 claim
        test.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 1);
        const balanceBefore: NEAR = (await alice.balance()).total;
        //During the callback this number is decreased
        await voteOnBounty(root, dao, proposalId + 1);

        //Proposal should be approved
        let { status } = await dao.view('get_proposal', { id: proposalId + 1 });
        test.is(status, 'Approved');

        //During the callback the number bounty_claims_count should decrease
        const balanceAfter: NEAR = (await alice.balance()).total;
        test.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 0);
        test.assert(balanceBefore.lt(balanceAfter));
    },
);

workspace.test('Callback transfer', async (test, { alice, root, dao }) => {
    const user1 = await root.createAccount('user1');
    // Fail transfer by transfering to non-existent accountId
    let transferId: number = await user1.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'give me tokens',
                kind: {
                    Transfer: {
                        token_id: '',
                        receiver_id: 'broken_id',
                        amount: toYocto('1'),
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );
    let user1Balance = (await user1.balance()).total;
    await voteApprove(root, dao, transferId);
    let { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Failed');
    test.assert((await user1.balance()).total.eq(user1Balance)); // no bond returns on fail

    // now we transfer to real accountId
    transferId = await user1.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'give me tokens',
                kind: {
                    Transfer: {
                        token_id: '',
                        receiver_id: alice.accountId, // valid id this time
                        amount: toYocto('1'),
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );
    user1Balance = (await user1.balance()).total;
    await voteApprove(root, dao, transferId);
    ({ status } = await dao.view('get_proposal', { id: transferId }));
    test.is(status, 'Approved');
    test.assert((await user1.balance()).total.gt(user1Balance)); // returns bond
});

workspace.test('Callback function call', async (test, { alice, root, dao }) => {
    const testToken = await initTestToken(root);
    let transferId: number = await root.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'give me tokens',
                kind: {
                    FunctionCall: {
                        receiver_id: testToken.accountId,
                        actions: [
                            {
                                method_name: 'fail',
                                args: Buffer.from('bad args').toString(
                                    'base64',
                                ),
                                deposit: toYocto('1'),
                                gas: tGas(10),
                            },
                        ],
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: transferId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(200),
        },
    );
    let { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Failed');

    transferId = await root.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'give me tokens',
                kind: {
                    FunctionCall: {
                        receiver_id: testToken.accountId,
                        actions: [
                            {
                                method_name: 'mint',
                                args: Buffer.from(
                                    '{"account_id": "' +
                                        alice.accountId +
                                        '", "amount": "10"}',
                                ).toString('base64'),
                                deposit: '0',
                                gas: tGas(10),
                            },
                            {
                                method_name: 'burn',
                                args: Buffer.from(
                                    '{"account_id": "' +
                                        alice.accountId +
                                        '", "amount": "10"}',
                                ).toString('base64'),
                                deposit: '0',
                                gas: tGas(10),
                            },
                        ],
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: transferId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(200),
        },
    );
    ({ status } = await dao.view('get_proposal', { id: transferId }));
    test.is(status, 'Approved');
});

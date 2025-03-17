import {
    BN,
    NearAccount,
    captureError,
    toYocto,
    tGas,
    DEFAULT_FUNCTION_CALL_GAS,
    Gas,
    NEAR,
} from 'near-workspaces';
import {
    initStaking,
    initTestToken,
    STORAGE_PER_BYTE,
    initWorkspace,
    getProposalKind,
    Proposal,
    DAO_WASM_BYTES,
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


const test = initWorkspace();

test('Remove blob', async (t) => {
    const { root, dao, alice } = t.context.accounts;
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
    t.regex(err, /ERR_NO_BLOB/);

    // Can only be called by the original storer
    err = await captureError(async () =>
        alice.call(dao, 'remove_blob', {
            hash: hash,
        }),
    );
    t.regex(err, /ERR_INVALID_CALLER/);

    // blob is removed with payback
    const rootAmountBeforeRemove = (await root.balance()).total;
    await root.call(dao, 'remove_blob', {
        hash: hash,
    });
    const rootAmountAfterRemove = (await root.balance()).total;
    t.false(await dao.view('has_blob', { hash: hash }));
    t.assert(rootAmountAfterRemove.gt(rootAmountBeforeRemove));
});

test(
    'Callback for BountyDone with NEAR token',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        //During the callback the number bounty_claims_count should decrease
        const proposalId = await proposeBountyWithNear(alice, dao);
        await voteOnBounty(root, dao, proposalId);
        await claimBounty(alice, dao, proposalId);
        await doneBounty(alice, alice, dao, proposalId);
        //Before the bounty is done there is 1 claim
        t.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 1);
        const balanceBefore: NEAR = (await alice.balance()).total;
        //During the callback this number is decreased
        await voteOnBounty(root, dao, proposalId + 1);
        const balanceAfter: NEAR = (await alice.balance()).total;
        t.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 0);
        t.assert(balanceBefore.lt(balanceAfter));
    },
);

test(
    'Callback for BountyDone ft token fail',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
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
        let { status }: Proposal = await dao.view('get_proposal', {
            id: proposalIdFail + 1,
        });
        t.is(status, 'Failed');
    },
);

test(
    'Callback for BountyDone ft token',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
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
        t.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 1);
        const balanceBefore: NEAR = (await alice.balance()).total;
        //During the callback this number is decreased
        await voteOnBounty(root, dao, proposalId + 1);

        //Proposal should be approved
        let { status } : Proposal = await dao.view('get_proposal', { id: proposalId + 1 });
        t.is(status, 'Approved');

        //During the callback the number bounty_claims_count should decrease
        const balanceAfter: NEAR = (await alice.balance()).total;
        t.is(await dao.view('get_bounty_number_of_claims', { id: 0 }), 0);
        t.assert(balanceBefore.lt(balanceAfter));
    },
);

test('Callback transfer', async (t) => {
    const { alice, root, dao } = t.context.accounts;
    const user1 = await root.createSubAccount('user1');
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
    let { status } : Proposal  = await dao.view('get_proposal', { id: transferId });
    t.is(status, 'Failed');
    t.assert((await user1.balance()).total.eq(user1Balance)); // no bond returns on fail

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
    ({ status } = await dao.view('get_proposal', { id: transferId }) as Proposal );
    t.is(status, 'Approved');
    t.assert((await user1.balance()).total.gt(user1Balance)); // returns bond
});

test('Callback function call', async (t) => {
    const { alice, root, dao } = t.context.accounts;
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
            proposal: await getProposalKind(dao, transferId),
        },
        {
            gas: tGas(200),
        },
    );
    let { status } : Proposal = await dao.view('get_proposal', { id: transferId });
    t.is(status, 'Failed');

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
            proposal: await getProposalKind(dao, transferId),
        },
        {
            gas: tGas(200),
        },
    );
    ({ status } = await dao.view('get_proposal', { id: transferId }) as Proposal);
    t.is(status, 'Approved');
});

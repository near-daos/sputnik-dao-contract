import {
    toYocto,
    NearAccount,
    captureError,
    BN,
    NEAR,
    ONE_NEAR,
    tGas,
} from 'near-workspaces';

import {
    initTestToken,
    initStaking,
    setStakingId,
    voteApprove,
    Proposal,
    initWorkspace,
    getProposalKind,
    normalizePolicy
} from './utils';

const test = initWorkspace();

test('basic', async (t) => {
    const { alice, root, dao } = t.context.accounts;
    t.true(await alice.exists());
    t.true(await root.exists());
    t.true(await dao.exists());
    t.log(await dao.view('get_config'));
});

test(
    'add_proposal fails in case of insufficient deposit',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        t.is(await dao.view('get_last_proposal_id'), 0);
        const config = {
            name: 'sputnikdao',
            purpose: 'testing',
            metadata: '',
        };
        //Try adding a proposal with 0.999... near
        let err = await captureError(
            async () =>
                await alice.call(
                    dao,
                    'add_proposal',
                    {
                        proposal: {
                            description: 'rename the dao',
                            kind: {
                                ChangeConfig: {
                                    config,
                                },
                            },
                        },
                    },
                    { attachedDeposit: new BN(toYocto('1')).subn(1) },
                ),
        );

        t.log(err.toString());
        t.true(err.includes('ERR_MIN_BOND'));
        //the proposal did not count
        t.is(await dao.view('get_last_proposal_id'), 0);

        //Checks that the same proposal doesn't fail
        //if the deposit is at least 1 near
        await alice.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'rename the dao',
                    kind: {
                        ChangeConfig: {
                            config,
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );
        t.is(await dao.view('get_last_proposal_id'), 1);

        let new_proposal: any = await dao.view('get_proposal', { id: 0 });

        t.log(new_proposal);
        t.is(new_proposal.description, 'rename the dao');
        t.is(new_proposal.proposer, 'alice.test.near');
        t.is(new_proposal.status, 'InProgress');

        t.truthy(new_proposal.kind.ChangeConfig);
        t.is(new_proposal.kind.ChangeConfig.config.name, 'sputnikdao');
        //same config as we did not execute that proposal
        t.deepEqual(await dao.view('get_config'), {
            name: 'sputnik',
            purpose: 'testing',
            metadata: '',
        });
    },
);

test(
    'Bob can not add proposals',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const bob = await root.createSubAccount('bob');

        //First we change a policy so that Bob can't add proposals
        const period = new BN('1000000000')
            .muln(60)
            .muln(60)
            .muln(24)
            .muln(7)
            .toString();
        const newPolicy = {
            roles: [
                {
                    name: 'all',
                    kind: {
                        Group: [root.accountId, alice.accountId],
                    },
                    permissions: ['*:VoteApprove', '*:AddProposal'],
                    vote_policy: {},
                },
            ],
            default_vote_policy: {
                weight_kind: 'TokenWeight',
                quorum: new BN('1').toString(),
                threshold: '5',
            },
            proposal_bond: toYocto('1'),
            proposal_period: period,
            bounty_bond: toYocto('1'),
            bounty_forgiveness_period: period,
        };
        let id: number = await bob.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description:
                        'change to a new policy, so that bob can not add a proposal',
                    kind: {
                        ChangePolicy: {
                            policy: newPolicy,
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );
        await voteApprove(root, dao, id);

        //Chrck that only those with a permission can add the proposal
        let errorString = await captureError(
            async () =>
                await bob.call(
                    dao,
                    'add_proposal',
                    {
                        proposal: {
                            description: 'change to a new policy',
                            kind: {
                                ChangePolicy: {
                                    policy: newPolicy,
                                },
                            },
                        },
                    },
                    { attachedDeposit: toYocto('1') },
                ),
        );
        t.regex(errorString, /ERR_PERMISSION_DENIED/);
    },
);

test('Proposal ChangePolicy', async (t) => {
    const { alice, root, dao } = t.context.accounts;
    t.deepEqual(
        await dao.view('get_proposals', { from_index: 0, limit: 10 }),
        [],
    );

    //Check that we can't change policy to a policy unless it's VersionedPolicy::Current
    let policy = [root.accountId];
    let errorString = await captureError(
        async () =>
            await alice.call(
                dao,
                'add_proposal',
                {
                    proposal: {
                        description: 'change the policy',
                        kind: {
                            ChangePolicy: {
                                policy,
                            },
                        },
                    },
                },
                { attachedDeposit: toYocto('1') },
            ),
    );
    t.regex(errorString, /ERR_INVALID_POLICY/);

    //Check that we can change to a correct policy
    const period = new BN('1000000000')
        .muln(60)
        .muln(60)
        .muln(24)
        .muln(7)
        .toString();
    const correctPolicy = {
        roles: [
            {
                name: 'all',
                kind: {
                    Group: [root.accountId, alice.accountId],
                },
                permissions: ['*:AddProposal', '*:VoteApprove'],
                vote_policy: {},
            },
        ],
        default_vote_policy: {
            weight_kind: 'TokenWeight',
            quorum: new BN('1').toString(),
            threshold: '5',
        },
        proposal_bond: toYocto('1'),
        proposal_period: period,
        bounty_bond: toYocto('1'),
        bounty_forgiveness_period: period,
    };
    let id: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'change to a new correct policy',
                kind: {
                    ChangePolicy: {
                        policy: correctPolicy,
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );

    //Number of proposals = 1
    t.is(await dao.view('get_last_proposal_id'), 1);
    //Check that the proposal is added to the list of proposals
    let proposals = await dao.view('get_proposals', {
        from_index: 0,
        limit: 10,
    });
    let realProposal = {
        id: 0,
        proposer: alice.accountId,
        description: 'change to a new correct policy',
        kind: { ChangePolicy: { policy: correctPolicy } },
        status: 'InProgress',
        vote_counts: {},
        votes: {},
    };
    t.is(proposals[0].id, realProposal.id);
    t.is(proposals[0].proposer, realProposal.proposer);
    t.is(proposals[0].description, realProposal.description);
    t.is(proposals[0].status, realProposal.status);
    t.deepEqual(proposals[0].vote_counts, realProposal.vote_counts);
    t.deepEqual(proposals[0].votes, realProposal.votes);
    t.deepEqual(proposals[0].kind, realProposal.kind);

    //After voting on the proposal it is Approved
    await voteApprove(root, dao, id);

    t.deepEqual(
        (await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0]
            .vote_counts,
        { council: [1, 0, 0] },
    );
    t.is(
        (await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0]
            .status,
        'Approved',
    );

    //Check that the policy is changed
    
    t.deepEqual(normalizePolicy(await dao.view('get_policy')), normalizePolicy(correctPolicy));
});

test('Proposal Transfer', async (t) => {
    const { alice, root, dao } = t.context.accounts;
    let errorString = await captureError(
        async () =>
            await root.call(
                dao,
                'add_proposal',
                {
                    proposal: {
                        description:
                            'can not use transfer without wrong token_id and msg',
                        kind: {
                            Transfer: {
                                token_id: '',
                                receiver_id: alice.accountId,
                                amount: toYocto('1'),
                                msg: 'some msg',
                            },
                        },
                    },
                },
                {
                    attachedDeposit: toYocto('1'),
                },
            ),
    );
    t.regex(errorString, /ERR_BASE_TOKEN_NO_MSG/);

    const transferId: number = await root.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'transfer 1 yocto',
                kind: {
                    Transfer: {
                        token_id: '',
                        receiver_id: alice,
                        amount: toYocto('1'),
                    },
                },
            },
        },
        { attachedDeposit: toYocto('1') },
    );
    const initBalance: NEAR = (await alice.balance()).total;
    await voteApprove(root, dao, transferId);
    const balance: NEAR = (await alice.balance()).total;
    t.deepEqual(balance, initBalance.add(ONE_NEAR));
});

test(
    'Proposal SetStakingContract',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const testToken = await initTestToken(root);
        const staking = await initStaking(root, dao, testToken);
        await setStakingId(root, dao, staking);

        t.is(await dao.view('get_staking_contract'), staking.accountId);

        let errorString = await captureError(
            async () => await setStakingId(root, dao, staking),
        );
        t.regex(errorString, /ERR_STAKING_CONTRACT_CANT_CHANGE/);
    },
);

test(
    'Voting is only allowed for councils',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const config = {
            name: 'sputnikdao',
            purpose: 'testing',
            metadata: '',
        };
        //add_proposal returns new proposal id
        const id: number = await alice.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'rename the dao',
                    kind: {
                        ChangeConfig: {
                            config,
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );

        //Check that voting is not allowed for non councils
        //Here alice tries to vote for her proposal but she is not a council and has no permission to vote.
        const err = await captureError(
            async () => await voteApprove(alice, dao, id),
        );
        t.log(err);
        t.true(err.includes('ERR_PERMISSION_DENIED'));

        let proposal: any = await dao.view('get_proposal', { id });
        t.log(proposal);
        t.is(proposal.status, 'InProgress');

        //Check that voting is allowed for councils
        //council (root) votes on alice's promise
        const res = await voteApprove(root, dao, id);
        proposal = await dao.view('get_proposal', { id });
        t.log(res);
        t.log(proposal);
        t.is(proposal.status, 'Approved');

        // proposal approved so now the config is equal to what alice did propose
        t.deepEqual(await dao.view('get_config'), config);
    },
);

test(
    'act_proposal should include correct kind',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const config = {
            name: 'sputnikdao',
            purpose: 'testing',
            metadata: '',
        };
        const wrong_config = {
            name: 'sputnikdao_fake',
            purpose: 'testing',
            metadata: '',
        };
        //add_proposal returns new proposal id
        const id: number = await alice.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'rename the dao',
                    kind: {
                        ChangeConfig: {
                            config,
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );

        //Check that act_proposal is not allowed with wrong kind included
        const err = await captureError(
            async () => await root.call(
                dao,
                'act_proposal',
                {
                    id: id,
                    action: 'VoteApprove',
                    proposal: {
                        ChangeConfig: {
                            config: wrong_config,
                        },
                    },
                },
                {
                    gas: tGas(100),
                },
            ),
        );
        t.log(err);
        t.true(err.includes('ERR_WRONG_KIND'));

        let proposal: any = await dao.view('get_proposal', { id });
        t.log(proposal);
        t.is(proposal.status, 'InProgress');
    },
);


// If the number of votes in the group has changed (new members has been added)
//  the proposal can lose it's approved state.
//  In this case new proposal needs to be made, this one should expire
test(
    'Proposal group changed during voting',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const transferId: number = await root.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'give me tokens',
                    kind: {
                        Transfer: {
                            token_id: '',
                            receiver_id: alice,
                            amount: toYocto('1'),
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );

        const addMemberToRoleId: number = await root.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'add alice',
                    kind: {
                        AddMemberToRole: {
                            member_id: alice,
                            role: 'council',
                        },
                    },
                },
            },
            { attachedDeposit: toYocto('1') },
        );
        await voteApprove(root, dao, addMemberToRoleId);
        await voteApprove(root, dao, transferId);
        const { status } : Proposal = await dao.view('get_proposal', { id: transferId });
        t.is(status, 'InProgress');
    },
);

test('Proposal transfer ft', async (t) => {
    const { alice, root, dao } = t.context.accounts;
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
    const transferId: number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'transfer tokens to me',
                kind: {
                    Transfer: {
                        token_id: testToken.accountId,
                        receiver_id: alice.accountId,
                        amount: '10',
                    },
                },
            },
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
    await voteApprove(root, dao, transferId);
    const { status } : Proposal  = await dao.view('get_proposal', { id: transferId });
    t.is(status, 'Approved');
});

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
    ({ status } = await dao.view('get_proposal', { id: transferId }) as Proposal);
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
    let { status } : Proposal  = await dao.view('get_proposal', { id: transferId });
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

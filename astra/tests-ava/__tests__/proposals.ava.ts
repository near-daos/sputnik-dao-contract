import {
    toYocto,
    NearAccount,
    captureError,
    BN,
    NEAR,
    ONE_NEAR,
    tGas,
} from 'near-workspaces-ava';

import {
    workspace,
    initTestToken,
    initStaking,
    setStakingId,
    workspaceWithoutInit,
    voteApprove,
} from './utils';

workspace.test('basic', async (test, { alice, root, dao }) => {
    test.true(await alice.exists());
    test.true(await root.exists());
    test.true(await dao.exists());
    test.log(await dao.view('get_config'));
});

workspace.test(
    'add_proposal fails in case of insufficient deposit',
    async (test, { alice, root, dao }) => {
        test.is(await dao.view('get_last_proposal_id'), 0);
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

        test.log(err.toString());
        test.true(err.includes('ERR_MIN_BOND'));
        //the proposal did not count
        test.is(await dao.view('get_last_proposal_id'), 0);

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
        test.is(await dao.view('get_last_proposal_id'), 1);

        let new_proposal: any = await dao.view('get_proposal', { id: 0 });

        test.log(new_proposal);
        test.is(new_proposal.description, 'rename the dao');
        test.is(new_proposal.proposer, 'alice.test.near');
        test.is(new_proposal.status, 'InProgress');

        test.truthy(new_proposal.kind.ChangeConfig);
        test.is(new_proposal.kind.ChangeConfig.config.name, 'sputnikdao');
        //same config as we did not execute that proposal
        test.deepEqual(await dao.view('get_config'), {
            name: 'sputnik',
            purpose: 'testing',
            metadata: '',
        });
    },
);

workspace.test(
    'Bob can not add proposals',
    async (test, { alice, root, dao }) => {
        const bob = await root.createAccount('bob');

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
        test.regex(errorString, /ERR_PERMISSION_DENIED/);
    },
);

workspace.test('Proposal ChangePolicy', async (test, { alice, root, dao }) => {
    test.deepEqual(
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
    test.regex(errorString, /ERR_INVALID_POLICY/);

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
    test.is(await dao.view('get_last_proposal_id'), 1);
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
    test.is(proposals[0].id, realProposal.id);
    test.is(proposals[0].proposer, realProposal.proposer);
    test.is(proposals[0].description, realProposal.description);
    test.is(proposals[0].status, realProposal.status);
    test.deepEqual(proposals[0].vote_counts, realProposal.vote_counts);
    test.deepEqual(proposals[0].votes, realProposal.votes);
    test.deepEqual(proposals[0].kind, realProposal.kind);

    //After voting on the proposal it is Approved
    await voteApprove(root, dao, id);

    test.deepEqual(
        (await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0]
            .vote_counts,
        { council: [1, 0, 0] },
    );
    test.is(
        (await dao.view('get_proposals', { from_index: 0, limit: 10 }))[0]
            .status,
        'Approved',
    );

    //Check that the policy is changed
    test.deepEqual(await dao.view('get_policy'), correctPolicy);
});

workspace.test('Proposal Transfer', async (test, { alice, root, dao }) => {
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
    test.regex(errorString, /ERR_BASE_TOKEN_NO_MSG/);

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
    test.deepEqual(balance, initBalance.add(ONE_NEAR));
});

workspace.test(
    'Proposal SetStakingContract',
    async (test, { alice, root, dao }) => {
        const testToken = await initTestToken(root);
        const staking = await initStaking(root, dao, testToken);
        await setStakingId(root, dao, staking);

        test.is(await dao.view('get_staking_contract'), staking.accountId);

        let errorString = await captureError(
            async () => await setStakingId(root, dao, staking),
        );
        test.regex(errorString, /ERR_STAKING_CONTRACT_CANT_CHANGE/);
    },
);

workspace.test(
    'Voting is only allowed for councils',
    async (test, { alice, root, dao }) => {
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
        test.log(err);
        test.true(err.includes('ERR_PERMISSION_DENIED'));

        let proposal: any = await dao.view('get_proposal', { id });
        test.log(proposal);
        test.is(proposal.status, 'InProgress');

        //Check that voting is allowed for councils
        //council (root) votes on alice's promise
        const res = await voteApprove(root, dao, id);
        proposal = await dao.view('get_proposal', { id });
        test.log(res);
        test.log(proposal);
        test.is(proposal.status, 'Approved');

        // proposal approved so now the config is equal to what alice did propose
        test.deepEqual(await dao.view('get_config'), config);
    },
);

// If the number of votes in the group has changed (new members has been added)
//  the proposal can lose it's approved state.
//  In this case new proposal needs to be made, this one should expire
workspace.test(
    'Proposal group changed during voting',
    async (test, { alice, root, dao }) => {
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
        const { status } = await dao.view('get_proposal', { id: transferId });
        test.is(status, 'InProgress');
    },
);

workspaceWithoutInit.test(
    'Proposal action types',
    async (test, { alice, root, dao }) => {
        const user1 = await root.createAccount('user1');
        const user2 = await root.createAccount('user2');
        const user3 = await root.createAccount('user3');
        const period = new BN('1000000000')
            .muln(60)
            .muln(60)
            .muln(24)
            .muln(7)
            .toString();
        const policy = {
            roles: [
                {
                    name: 'council',
                    kind: {
                        Group: [
                            alice.accountId,
                            user1.accountId,
                            user2.accountId,
                            user3.accountId,
                        ],
                    },
                    permissions: ['*:*'],
                    vote_policy: {},
                },
            ],
            default_vote_policy: {
                weight_kind: 'RoleWeight',
                quorum: new BN('0').toString(),
                threshold: [1, 2],
            },
            proposal_bond: toYocto('1'),
            proposal_period: period,
            bounty_bond: toYocto('1'),
            bounty_forgiveness_period: period,
        };

        let config = { name: 'sputnik', purpose: 'testing', metadata: '' };

        await root.call(dao, 'new', { config, policy });

        let proposalId = await alice.call(
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

        // Remove proposal works
        await alice.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'RemoveProposal',
        });
        let err = await captureError(async () =>
            dao.view('get_proposal', { id: proposalId }),
        );
        test.regex(err, /ERR_NO_PROPOSAL/);

        err = await captureError(async () =>
            alice.call(dao, 'act_proposal', {
                id: proposalId,
                action: 'VoteApprove',
            }),
        );
        test.regex(err, /ERR_NO_PROPOSAL/);

        proposalId = await alice.call(
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

        err = await captureError(async () =>
            alice.call(dao, 'act_proposal', {
                id: proposalId,
                action: 'AddProposal',
            }),
        );
        test.regex(err, /ERR_WRONG_ACTION/);

        // Check if every vote counts
        await user1.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
        });
        await user2.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteReject',
        });
        await alice.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteRemove',
        });
        {
            const { vote_counts, votes } = await dao.view('get_proposal', {
                id: proposalId,
            });
            test.deepEqual(vote_counts.council, [1, 1, 1]);
            test.deepEqual(votes, {
                [alice.accountId]: 'Remove',
                [user1.accountId]: 'Approve',
                [user2.accountId]: 'Reject',
            });
        }

        // Finalize proposal will panic if not exired or failed
        err = await captureError(async () =>
            alice.call(dao, 'act_proposal', {
                id: proposalId,
                action: 'Finalize',
            }),
        );
        test.regex(err, /ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED/);
    },
);

workspace.test('Proposal transfer ft', async (test, { alice, root, dao }) => {
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
    const { status } = await dao.view('get_proposal', { id: transferId });
    test.is(status, 'Approved');
});

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

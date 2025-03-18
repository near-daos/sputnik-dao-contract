import {
    BN,
    NearAccount,
    captureError,
    toYocto,
    tGas,
    DEFAULT_FUNCTION_CALL_GAS,
} from 'near-workspaces';
import {
    initStaking,
    initTestToken,
    STORAGE_PER_BYTE,
    registerAndDelegate,
    setStakingId,
    initWorkspace,
    Proposal,
    getProposalKind,
    DAO_WASM_BYTES,
} from './utils';

// Set up workspace without initializing DAO contract
const test = initWorkspace({ skipInit: true });

// -- Upgrade --
test(
    'Upgrade self negative',
    async (t) => {
        const { root, dao } = t.context.accounts;
        const config = { name: 'sputnik', purpose: 'testing', metadata: '' };

        // NOT INITIALIZED
        let err = await captureError(async () =>
            root.call(dao, 'store_blob', DAO_WASM_BYTES, {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            }),
        );
        t.regex(err, /ERR_CONTRACT_IS_NOT_INITIALIZED/);

        // Initializing contract
        await root.call(dao, 'new', { config, policy: [root.accountId] });

        // not enough deposit
        err = await captureError(async () =>
            root.call(dao, 'store_blob', DAO_WASM_BYTES, {
                attachedDeposit: toYocto('1'),
                gas: tGas(300),
            }),
        );
        t.regex(err, /ERR_NOT_ENOUGH_DEPOSIT/);

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
        t.regex(err, /ERR_ALREADY_EXISTS/);
    },
);

// -- Proposal --
test(
    'Proposal action types',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const user1 = await root.createSubAccount('user1');
        const user2 = await root.createSubAccount('user2');
        const user3 = await root.createSubAccount('user3');
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

        let proposalId: number = await alice.call(
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
            proposal: await getProposalKind(dao, proposalId),
        });
        let err = await captureError(async () =>
            dao.view('get_proposal', { id: proposalId }),
        );
        t.regex(err, /ERR_NO_PROPOSAL/);

        err = await captureError(async () =>
            alice.call(dao, 'act_proposal', {
                id: proposalId,
                action: 'VoteApprove',
                proposal: await getProposalKind(dao, proposalId),
            }),
        );
        t.regex(err, /ERR_NO_PROPOSAL/);

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
                proposal: await getProposalKind(dao, proposalId),
            }),
        );
        t.regex(err, /ERR_WRONG_ACTION/);

        // Check if every vote counts
        const proposal_kind = await getProposalKind(dao, proposalId);
        await user1.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
            proposal: proposal_kind,
        });
        await user2.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteReject',
            proposal: proposal_kind,
        });
        await alice.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteRemove',
            proposal: proposal_kind,
        });
        {
            const { vote_counts, votes } = await dao.view('get_proposal', {
                id: proposalId,
            }) as any;
            t.deepEqual(vote_counts.council, [1, 1, 1]);
            t.deepEqual(votes, {
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
                proposal: proposal_kind,
            }),
        );
        t.regex(err, /ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED/);
    },
);

// -- Policy --
test(
    'Testing policy TokenWeight',
    async (t) => {
        const { alice, root, dao } = t.context.accounts;
        const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
        const bob = await root.createSubAccount('bob');
        const period = new BN('1000000000')
            .muln(60)
            .muln(60)
            .muln(24)
            .muln(7)
            .toString();
        const testToken = await initTestToken(root);
        const staking = await initStaking(root, dao, testToken);
        await root.call(dao, 'new', { config, policy: [root.accountId] });
        await setStakingId(root, dao, staking);

        const policy = {
            roles: [
                {
                    name: 'all',
                    kind: { Group: [alice.accountId, bob.accountId] }, // fails with kind: "Everyone" need to investigate
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

        let proposalId: number = await alice.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'test',
                    kind: { ChangePolicy: { policy } },
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

        // Setting up a new config
        const new_config = {
            name: 'new dao wohoo',
            purpose: 'testing',
            metadata: '',
        };
        await registerAndDelegate(dao, staking, alice, new BN('1'));
        await registerAndDelegate(dao, staking, bob, new BN('4'));
        proposalId = await alice.call(
            dao,
            'add_proposal',
            {
                proposal: {
                    description: 'test',
                    kind: {
                        ChangeConfig: {
                            config: new_config,
                        },
                    },
                },
            },
            {
                attachedDeposit: toYocto('1'),
            },
        );
        const proposal_kind = await getProposalKind(dao, proposalId);
        await alice.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
            proposal: proposal_kind,
        });
        await bob.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
            proposal: proposal_kind,
        });
        t.deepEqual(await dao.view('get_config'), new_config);
    },
);

test('Policy self-lock', async (t) => {
    const { alice, root, dao } = t.context.accounts;
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
    const period = new BN('1000000000')
        .muln(60)
        .muln(60)
        .muln(24)
        .muln(7)
        .toString();
    const policy = {
        roles: [
            {
                name: 'all',
                kind: { Group: [alice.accountId] },
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
    // 'staking_id' is not set, we can't delegate, so this contract got locked
    await root.call(dao, 'new', { config, policy });
    const proposalId = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: {
                    ChangePolicy: {
                        policy,
                    },
                },
            },
        },
        {
            attachedDeposit: toYocto('1'),
        },
    );
    await alice.call(dao, 'act_proposal', {
        id: proposalId,
        action: 'VoteApprove',
        proposal: {
            ChangePolicy: {
                policy,
            },
        },
    });
    let { status } : Proposal = await dao.view('get_proposal', { id: proposalId });
    t.is(status, 'InProgress');
});

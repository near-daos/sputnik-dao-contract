import {
    Workspace,
    BN,
    NearAccount,
    captureError,
    toYocto,
    tGas,
    DEFAULT_FUNCTION_CALL_GAS,
} from 'near-workspaces-ava';
import {
    initStaking,
    initTestToken,
    STORAGE_PER_BYTE,
    registerAndDelegate,
    setStakingId,
    workspaceWithoutInit as workspace,
} from './utils';

workspace.test(
    'Testing policy TokenWeight',
    async (test, { alice, root, dao }) => {
        const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
        const bob = await root.createAccount('bob');
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
        await alice.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
        });
        await bob.call(dao, 'act_proposal', {
            id: proposalId,
            action: 'VoteApprove',
        });
        test.deepEqual(await dao.view('get_config'), new_config);
    },
);

workspace.test('Policy self-lock', async (test, { alice, root, dao }) => {
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
    });
    let { status } = await dao.view('get_proposal', { id: proposalId });
    test.is(status, 'InProgress');
});

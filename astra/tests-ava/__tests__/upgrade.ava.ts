import { toYocto, tGas } from 'near-workspaces-ava';

import { workspaceWithFactory } from './utils';

// DAO v2 Upgrade flow:
// 1. add proposal for store_contract_self(get it approved)
// 2. add proposal for UpgradeSelf with hash of blob from #1(get it approved)
// 3. add proposal for remove_contract_self(get it approved)
// 4. Confirm DAO contract code_hash and returned balance

workspaceWithFactory.test('basic', async (test, { root, factory }) => {
    test.true(await root.exists());
    test.true(await factory.exists());
});

workspaceWithFactory.test(
    'Store DAO upgrade code in DAO via factory',
    async (test, { root, factory }) => {
        const config = {
            name: 'upgradedao',
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
                name: 'upgradedao',
                args: Buffer.from(JSON.stringify(params)).toString('base64'),
            },
            {
                attachedDeposit: toYocto('20'),
                gas: tGas(300),
            },
        );

        test.deepEqual(await factory.view('get_dao_list', {}), [
            'upgradedao.factory.test.near',
        ]);

        // 1. add proposal for store_contract_self(get it approved)
        // --------------------------------------------------------------------
        const six_near = toYocto('6');
        const default_code_hash = await factory.view('get_default_code_hash');

        let proposalId: number = await root.call(
            'upgradedao.factory.test.near',
            'get_last_proposal_id',
            {},
            { gas: tGas(300) },
        );
        test.is(proposalId, 0);

        const args = Buffer.from(
            `{ "code_hash": "${default_code_hash}" }`,
            'binary',
        ).toString('base64');

        const proposal = {
            proposal: {
                description: 'Store DAO upgrade contract code blob',
                kind: {
                    FunctionCall: {
                        receiver_id: `${factory.accountId}`,
                        actions: [
                            {
                                method_name: 'store_contract_self',
                                args: args,
                                deposit: six_near,
                                gas: tGas(220),
                            },
                        ],
                    },
                },
            },
        };

        await root.call(
            'upgradedao.factory.test.near',
            'add_proposal',
            proposal,
            {
                attachedDeposit: toYocto('1'),
                gas: tGas(300),
            },
        );

        proposalId = await root.call(
            'upgradedao.factory.test.near',
            'get_last_proposal_id',
            {},
            { gas: tGas(300) },
        );
        test.is(proposalId, 1);

        let new_proposal: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 0 },
            { gas: tGas(300) },
        );

        test.log(new_proposal);
        test.is(
            new_proposal.description,
            'Store DAO upgrade contract code blob',
        );
        test.is(new_proposal.proposer, 'test.near');
        test.is(new_proposal.status, 'InProgress');
        test.truthy(new_proposal.kind.FunctionCall);
        test.is(
            new_proposal.kind.FunctionCall.receiver_id,
            `${factory.accountId}`,
        );

        await root.call(
            'upgradedao.factory.test.near',
            'act_proposal',
            { id: 0, action: 'VoteApprove' },
            { gas: tGas(300) },
        );

        let passed_proposal_0: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 0 },
            { gas: tGas(300) },
        );
        test.log(passed_proposal_0);
        test.is(passed_proposal_0.status, 'Approved');

        // 2. add proposal for UpgradeSelf with hash of blob from #1(get it approved)
        // --------------------------------------------------------------------
        const proposalUpgradeSelf = {
            proposal: {
                description: 'Upgrade DAO contract using local code blob',
                kind: {
                    UpgradeSelf: {
                        hash: `${default_code_hash}`,
                    },
                },
            },
        };
        await root.call(
            'upgradedao.factory.test.near',
            'add_proposal',
            proposalUpgradeSelf,
            {
                attachedDeposit: toYocto('1'),
                gas: tGas(300),
            },
        );

        let new_proposal_1: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 1 },
            { gas: tGas(300) },
        );

        test.log(new_proposal_1);
        test.is(
            new_proposal_1.description,
            'Upgrade DAO contract using local code blob',
        );
        test.is(new_proposal_1.proposer, 'test.near');
        test.is(new_proposal_1.status, 'InProgress');
        test.truthy(new_proposal_1.kind.UpgradeSelf);

        await root.call(
            'upgradedao.factory.test.near',
            'act_proposal',
            { id: 1, action: 'VoteApprove' },
            { gas: tGas(300) },
        );

        let passed_proposal_1: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 1 },
            { gas: tGas(300) },
        );

        test.log(passed_proposal_1);
        test.is(passed_proposal_1.status, 'Approved');

        // 3. add proposal for remove_contract_self(get it approved)
        // --------------------------------------------------------------------
        const proposalRemoveContractBlob = {
            proposal: {
                description:
                    'Remove DAO upgrade contract local code blob via factory',
                kind: {
                    FunctionCall: {
                        receiver_id: `${factory.accountId}`,
                        actions: [
                            {
                                method_name: 'remove_contract_self',
                                args: Buffer.from(
                                    `{ "code_hash": "${default_code_hash}" }`,
                                    'binary',
                                ).toString('base64'),
                                deposit: '0',
                                gas: tGas(220),
                            },
                        ],
                    },
                },
            },
        };
        // console.log(
        //     'proposalRemoveContractBlob',
        //     JSON.stringify(proposalRemoveContractBlob)
        // );

        await root.call(
            'upgradedao.factory.test.near',
            'add_proposal',
            proposalRemoveContractBlob,
            {
                attachedDeposit: toYocto('1'),
                gas: tGas(300),
            },
        );

        let new_proposal_2: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 2 },
            { gas: tGas(300) },
        );

        test.log(new_proposal_2);
        test.is(
            new_proposal_2.description,
            'Remove DAO upgrade contract local code blob via factory',
        );
        test.is(new_proposal_2.proposer, 'test.near');
        test.is(new_proposal_2.status, 'InProgress');
        test.truthy(new_proposal_2.kind.FunctionCall);
        test.is(
            new_proposal_2.kind.FunctionCall.receiver_id,
            `${factory.accountId}`,
        );

        await root.call(
            'upgradedao.factory.test.near',
            'act_proposal',
            { id: 2, action: 'VoteApprove' },
            { gas: tGas(300) },
        );

        let passed_proposal_2: any = await root.call(
            'upgradedao.factory.test.near',
            'get_proposal',
            { id: 2 },
            { gas: tGas(300) },
        );

        test.log(passed_proposal_2);
        test.is(passed_proposal_2.status, 'Approved');

        // 4. Confirm DAO contract code_hash and returned balance
        // --------------------------------------------------------------------
        // TODO: Check if balance increased by 6 NEAR for refund
    },
);

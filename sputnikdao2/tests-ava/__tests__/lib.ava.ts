import { BN, NearAccount, captureError, toYocto, tGas, DEFAULT_FUNCTION_CALL_GAS, Gas, NEAR } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE, workspaceWithoutInit, workspaceWithFactory } from './utils';
import { voteApprove } from './utils';
import { DEADLINE, BOND, proposeBounty, proposeBountyWithNear, voteOnBounty, claimBounty, doneBounty } from './utils'
import * as fs from 'fs';

const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');

workspaceWithFactory.test('Upgrade self using factory', async (test, {
    root,
    factory
}) => {
    await factory.call(
        factory,
        'new', {}, {
            gas: tGas(300),
        }
    );

    const config = {
        name: 'testdao',
        purpose: 'to test',
        metadata: ''
    };
    const policy = [root.accountId];
    const params = {
        config,
        policy
    };

    await root.call(
        factory,
        'create', {
            name: 'testdao',
            args: Buffer.from(JSON.stringify(params)).toString('base64')
        }, {
            attachedDeposit: toYocto('10'),
            gas: tGas(300),
        }
    );

    test.deepEqual(await factory.view('get_dao_list', {}), ['testdao.factory.test.near']);
    const hash = await factory.view('get_default_code_hash', {});

    const result = await root
        .createTransaction('testdao.factory.test.near')
        .functionCall(
            'add_proposal', {
                proposal: {
                    description: 'proposal to test',
                    kind: {
                        "UpgradeSelf": {
                            hash: hash
                        }
                    }
                }
            }, {
                attachedDeposit: toYocto('1'),
            })
        .signAndSend();
    const proposalId = result.parseResult < number > ();
    test.is(proposalId, 0);

    await root
        .createTransaction('testdao.factory.test.near')
        .functionCall(
            'act_proposal', {
                id: proposalId,
                action: 'VoteApprove',
            }, {
                gas: tGas(300),
            })
        .signAndSend();
});

workspaceWithoutInit.test('Upgrade self negative', async (test, { root, dao }) => {
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };

    // NOT INITIALIZED
    let err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('200'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_CONTRACT_IS_NOT_INITIALIZED/);

    // Initializing contract
    await root.call(
        dao,
        'new',
        { config, policy: [root.accountId] },
    );

    // not enough deposit
    err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('1'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_NOT_ENOUGH_DEPOSIT/);

    await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();

    // Already exists
    err = await captureError(async () =>
        root
            .createTransaction(dao)
            .functionCall(
                'store_blob',
                DAO_WASM_BYTES,
                {
                    attachedDeposit: toYocto('200'),
                    gas: tGas(300),
                })
            .signAndSend()
    );
    test.regex(err, /ERR_ALREADY_EXISTS/);

});

workspace.test('Remove blob', async (test, { root, dao, alice }) => {
    const result = await root
        .createTransaction(dao)
        .functionCall(
            'store_blob',
            DAO_WASM_BYTES,
            {
                attachedDeposit: toYocto('200'),
                gas: tGas(300),
            })
        .signAndSend();

    const hash = result.parseResult<String>()
    
    // fails if hash is wrong
    let err = await captureError(async () =>
        root.call(
            dao,
            'remove_blob',
            {
                hash: "HLBiX51txizmQzZJMrHMCq4u7iEEqNbaJppZ84yW7628", // some_random hash
            }
        )
    );
    test.regex(err, /ERR_NO_BLOB/);

    // Can only be called by the original storer
    err = await captureError(async () =>
        alice.call(
            dao,
            'remove_blob',
            {
                hash: hash,
            }
        )
    );
    test.regex(err, /ERR_INVALID_CALLER/);

    // blob is removed with payback
    const rootAmountBeforeRemove = (await root.balance()).total
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
    const rootAmountAfterRemove = (await root.balance()).total
    test.false(await dao.view('has_blob', { hash: hash }));
    test.assert(rootAmountAfterRemove.gt(rootAmountBeforeRemove));
});
import { BN, NearAccount, captureError, toYocto, tGas, DEFAULT_FUNCTION_CALL_GAS, Gas, } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE, workspaceWithoutInit } from './utils';
import * as fs from 'fs';

const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');

workspace.test('Upgrade self', async (test, { root, dao }) => {
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
    const proposalId = await root.call(
        dao,
        'add_proposal',
        {
            proposal:
            {
                description: 'test',
                kind: { "UpgradeSelf": { hash: hash } }
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );


    const id: number = await dao.view('get_last_proposal_id');
    test.is(id, 1);

    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        },
        {
            gas: tGas(300), // attempt to subtract with overflow if not enough gas, maybe add some checks?
        }
    );

    test.is(await dao.view('version'), "2.0.0");

    const beforeBlobRemove = new BN(await dao.view('get_available_amount'));
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
    test.assert(
        new BN(await dao.view('get_available_amount')).gt(beforeBlobRemove)
    )
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
                attachedDeposit: toYocto('5'),
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
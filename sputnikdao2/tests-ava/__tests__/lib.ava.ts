import { BN, NearAccount, captureError, toYocto, tGas, DEFAULT_FUNCTION_CALL_GAS, } from 'near-workspaces-ava';
import { workspace, initStaking, initTestToken, STORAGE_PER_BYTE } from './utils';
import * as fs from 'fs';

workspace.test('Upgrade self', async (test, { root, dao }) => {
    const DAO_WASM_BYTES: Uint8Array = fs.readFileSync('../res/sputnikdao2.wasm');
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
    await root.call(
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
            id: 0,
            action: 'VoteApprove',
        },
        {
            gas: tGas(300), // attempt to subtract with overflow if not enough gas, maybe add some checks?
        }
    );
    
    test.is(await dao.view('version'), "2.0.0");
    await root.call(
        dao,
        'remove_blob',
        {
            hash: hash,
        }
    );
});

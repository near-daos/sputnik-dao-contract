import { Workspace, BN, NearAccount, captureError, toYocto, tGas, DEFAULT_FUNCTION_CALL_GAS } from 'near-workspaces-ava';
import { initStaking, initTestToken, STORAGE_PER_BYTE, registerAndDelegate, setStakingId } from './utils';

const workspace = Workspace.init(async ({ root }) => {
    const workspace = Workspace.init();

    const alice = await root.createAccount('alice');

    //for short let's call it just dao
    const dao = await root.createAndDeploy(
        'dao',
        '../res/sputnikdao2.wasm',
        {
            initialBalance: toYocto('200'),
        }
    );
    return { alice, dao };
});

// FINISH THIS TEST
workspace.test('Testing policy TokenWeight', async (test, { alice, root, dao }) => {
    const config = { name: 'sputnik', purpose: 'testing', metadata: '' };
    const period = new BN('1000000000').muln(60).muln(60).muln(24).muln(7).toString();
    const testToken = await initTestToken(root);
    const staking = await initStaking(root, dao, testToken);
    await root.call(
        dao,
        'new',
        { config, policy: [root.accountId] },
    );
    await setStakingId(root, dao, staking);

    const policy =
    {
        roles: [
            {
                name: "all",
                kind: "Everyone",
                permissions: ["*:*"],
                vote_policy: {}
            }
        ],
        default_vote_policy:
        {
            weight_kind: "TokenWeight",
            quorum: new BN('0').toString(),
            threshold: '0',
        },
        proposal_bond: toYocto('1'),
        proposal_period: period,
        bounty_bond: toYocto('1'),
        bounty_forgiveness_period: period,
    };

    let proposalId:number = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: {'ChangePolicy': {policy}},
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    console.log(await dao.view('get_proposal', {id: proposalId}));

    await registerAndDelegate(dao, staking, alice, new BN('1'));
    await registerAndDelegate(dao, staking, root, new BN('1'));
    proposalId = await alice.call(
        dao,
        'add_proposal',
        {
            proposal: {
                description: 'test',
                kind: {
                    'ChangeConfig': {config: 
                    {
                        name: "new dao wohoo",
                        purpose: "testing",
                        metadata: '',
                    }}},
            }
        },
        {
            attachedDeposit: toYocto('1'),
        }
    );
    await alice.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    await root.call(
        dao,
        'act_proposal',
        {
            id: proposalId,
            action: 'VoteApprove',
        }
    );
    console.log(await dao.view('get_proposal', {id: proposalId}));

    console.log(await dao.view('get_policy'));
    console.log(await dao.view('get_config'));
});
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Workspace = void 0;
const os = __importStar(require("os"));
const container_1 = require("./container");
const utils_1 = require("./utils");
/**
 * The main interface to near-workspaces. Create a new workspace instance with {@link Workspace.init}, then run code using {@link Workspace.fork}.
 *
 * @example
 * // Run multiple routines on testnet simultaneously
 * const workspace = Workspace.init({
 *   network: 'testnet', // Can also set the network using the NEAR_WORKSPACES_NETWORK environment variable
 *   rootAccount: 'me.testnet',
 * });
 * await Promise.all([
 *   workspace.fork(async ({root}) => {
 *     await root.call('some-contract.testnet', 'some_method', { a: 1, b: 2 });
 *   }),
 *   workspace.fork(async ({root}) => {
 *     await root.call('some-other-contract.testnet', 'some_method', { a: 2, b: 3 });
 *   }),
 * ]);
 *
 * @example
 * // Alternative syntax for the above
 * Workspace.open({network: 'testnet', rootAccount: 'me.testnet'}, async ({root}) => {
 *   await Promise.all([
 *     root.call('some-contract.testnet', 'some_method', { a: 1, b: 2 }),
 *     root.call('some-other-contract.testnet', 'some_method', { a: 2, b: 3 }),
 *   ]);
 * });
 *
 * @example
 * const {Workspace, NEAR} from 'near-workspaces';
 * // Test contracts in local sandbox mode, creating initial state for each `workspace.fork`
 * const workspace = Workspace.init(async ({root}) => {
 *   // Create a subaccount of `root`, such as `alice.dev-account-123456.testnet`
 *   const alice = root.createAccount('alice');
 *   // Create a subaccount of `root`, deploy a contract to it, and call a method on that contract
 *   const contract = root.createAndDeploy('contract-account-name', '../path/to/contract.wasm', {
 *     method: 'init',
 *     args: {owner_id: root}
 *   });
 *   // Everything in this Workspace.init function will happen prior to each call of `workspace.fork`
 *   await alice.call(contract, 'some_registration_method', {}, {
 *     attachedDeposit: NEAR.parse('50 milliNEAR')
 *   });
 *   // Accounts returned from `Workspace.init` function will be available in `workspace.fork` calls
 *   return {alice, contract};
 * });
 * workspace.fork(async ({alice, contract, root}) => {
 *   await root.call(contract, 'some_change_method', {account_id: alice});
 *   console.log({
 *     valueForRoot: await contract.view('some_view_method', {account_id: root});
 *     valueForAlice: await contract.view('some_view_method', {account_id: alice});
 *   });
 * });
 * workspace.fork(async ({alice, contract, root}) => {
 *   // This workspace does not call `some_change_method`
 *   console.log({
 *     valueForRoot: await contract.view('some_view_method', {account_id: root});
 *     valueForAlice: await contract.view('some_view_method', {account_id: alice});
 *   });
 * });
 */
class Workspace {
    constructor(workspaceContainerPromise) {
        this.ready = this.startWaiting(workspaceContainerPromise);
    }
    /**
     * Initialize a new workspace. In local sandbox mode, this will:
     *
     *   - Create a new local blockchain
     *   - Create the root account for that blockchain, available as `root`:
     *         Workspace.init(async => ({root}) => {...})
     *   - Execute any actions passed to the function
     *   - Shut down the newly created blockchain, but *save the data*
     *
     * In testnet mode, the same functionality is achieved via different means,
     * since all actions must occur on one blockchain instead of N.
     *
     * @param configOrFunction Either a configuration object or a function to run. Accounts returned from this function will be passed as arguments to subsequent `workspace.fork` calls.
     * @param f If configOrFunction is a config object, this must be a function to run
     * @returns an instance of the Workspace class, to be used as a starting point for forkd workspaces.
     */
    static init(configOrFunction = async () => ({}), f) {
        const { config, fn } = getConfigAndFn(configOrFunction, f);
        return new Workspace(container_1.WorkspaceContainer.create(config, fn));
    }
    static networkIsTestnet() {
        return this.getNetworkFromEnv() === 'testnet';
    }
    static networkIsSandbox() {
        return this.getNetworkFromEnv() === 'sandbox';
    }
    static getNetworkFromEnv() {
        return (0, utils_1.getNetworkFromEnv)();
    }
    /**
     * Sets up a connection to a network and executes the provided function.
     * Unlike `fork`, this will run the function once and not clean up after itself.
     * A rootAccount is required and if on testnet, will try to create account if it doesn't exist.
     * It also defaults to use your home directory's key store.
     *
     * @param config Config with the rootAccount argument required.
     * @param fn Function to run when connected.
     */
    static async open(config, fn) {
        const innerConfig = {
            init: false,
            rm: false,
            homeDir: os.homedir(),
            keyStore: (0, utils_1.homeKeyStore)(),
            ...config,
        };
        return (await container_1.WorkspaceContainer.create(innerConfig)).fork(fn);
    }
    async startWaiting(container) {
        this.container = await container;
    }
    /**
     * Run code in the context of a workspace initialized with `Workspace.init`.
     * In local sandbox mode, each `workspace.fork` will:
     *
     *   - start a new local blockchain
     *   - copy the state from the blockchain created in `Workspace.init`
     *   - get access to the accounts created in `Workspace.init` using the same variable names
     *   - keep all data isolated from other `workspace.fork` calls, so they can be run concurrently
     *   - shut down at the end, forgetting all new data created
     *
     * In testnet mode, the same functionality is achieved via different means,
     * since all actions must occur on one blockchain instead of N blockchains.
     *
     * @param fn code to run; has access to `root` and other accounts returned from function passed to `Workspace.init`. Example: `workspace.fork(async ({root, alice, bob}) => {...})`
     */
    async fork(fn) {
        await this.ready;
        const container = await this.container.createFrom();
        await container.fork(fn);
        return container;
    }
    /**
     * Like `fork`, but only runs when in local sandbox mode, not on testnet or mainnet. See `fork` docs for more info.
     *
     * @param fn code to run; has access to `root` and other accounts returned from function passed to `Workspace.init`. Example: `workspace.forkSandbox(async ({root, alice, bob}) => {...})`
     */
    async forkSandbox(fn) {
        await this.ready;
        if (this.container.config.network === 'sandbox') {
            return this.fork(fn);
        }
        return null;
    }
}
exports.Workspace = Workspace;
function getConfigAndFn(configOrFunction, f) {
    const type1 = typeof configOrFunction;
    const type2 = typeof f;
    if (type1 === 'function' && type2 === 'undefined') {
        // @ts-expect-error Type this|that not assignable to that
        return { config: {}, fn: configOrFunction };
    }
    if (type1 === 'object' && (type2 === 'function' || type2 === 'undefined')) {
        // @ts-expect-error Type this|that not assignable to that
        return { config: configOrFunction, fn: f };
    }
    throw new Error('Invalid arguments! '
        + 'Expected `(config, runFunction)` or just `(runFunction)`');
}
//# sourceMappingURL=workspace.js.map
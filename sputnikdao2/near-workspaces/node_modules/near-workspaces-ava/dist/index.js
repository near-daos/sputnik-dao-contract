"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Workspace = exports.ava = void 0;
const near_workspaces_1 = require("near-workspaces");
const ava_1 = __importDefault(require("ava")); // eslint-disable-line @typescript-eslint/no-duplicate-imports
exports.ava = ava_1.default;
__exportStar(require("near-workspaces"), exports);
/**
 * The main interface to near-workspace-ava. Create a new workspace instance with {@link Workspace.init}, then run tests using {@link Workspace.test}.
 *
 * @example
 * const {Workspace, NEAR, Gas} from 'near-workspace';
 * const workspace = Workspace.init(async ({root}) => {
 *   // Create a subaccount of `root`, such as `alice.sandbox` (get actual account ID with `alice.accountId`)
 *   const alice = root.createAccount('alice');
 *   // Create a subaccount of `root`, deploy a contract to it, and call a method on that contract
 *   const contract = root.createAndDeploy('contract-account-name', '../path/to/contract.wasm', {
 *     method: 'init',
 *     args: {owner_id: root}
 *   });
 *   // Everything in this Workspace.init function will happen prior to each call of `workspace.test`
 *   await alice.call(contract, 'some_registration_method', {}, {
 *     attachedDeposit: NEAR.parse('50 milliNEAR'),
 *     gas: Gas.parse('300Tgas'), // 300 Tgas is the max; 30 is the default
 *   });
 *   // Accounts returned from `Workspace.init` function will be available in `workspace.test` calls
 *   return {alice, contract};
 * });
 * workspace.test(async (test, {alice, contract, root}) => {
 *   await root.call(contract, 'some_change_method', {account_id: alice});
 *   // the `test` object comes from AVA, and has test assertions and other helpers
 *   test.is(
 *     await contract.view('some_view_method', {account_id: root});
 *     await contract.view('some_view_method', {account_id: alice});
 *   });
 * });
 * workspace.test(async (test, {alice, contract, root}) => {
 *   // This test does not call `some_change_method`
 *   test.not(
 *     await contract.view('some_view_method', {account_id: root});
 *     await contract.view('some_view_method', {account_id: alice});
 *   );
 * });
 */
class Workspace extends near_workspaces_1.Workspace {
    /**
     * Create a new workspace. In local sandbox mode, this will:
     *
     *   - Create a new local blockchain
     *   - Create the root account for that blockchain, available as `root`:
     *         Workspace.init(async => ({root}) => {...})
     *   - Execute any actions passed to the function
     *   - Shut down the newly created blockchain, but *save the data*
     *
     * In testnet mode, the same functionality is achieved via different means,
     * since all actions must occur on one blockchain instead of N blockchains.
     *
     * @param configOrFunction Either a configuration object or a function to run. Accounts returned from this function will be passed as arguments to subsequent `workspace.test` calls.
     * @param f If configOrFunction is a config object, this must be a function to run
     * @returns an instance of the Workspace class, which is used to run tests.
     */
    static init(configOrFunction = async () => ({}), f) {
        const workspace = near_workspaces_1.Workspace.init(configOrFunction, f);
        workspace.test = (description, fn = DEFAULT_TEST_FN) => {
            (0, ava_1.default)(description, async (t) => {
                await workspace.fork(async (args, workspace) => fn(t, args, workspace));
            });
        };
        return workspace;
    }
}
exports.Workspace = Workspace;
const DEFAULT_TEST_FN = () => {
    // Do nothing
};
//# sourceMappingURL=index.js.map
import { InitWorkspaceFn, Config, Workspace as RawWorkspace, AccountArgs, WorkspaceContainerInterface } from 'near-workspaces';
import * as ava from 'ava';
import test from 'ava';
export * from 'near-workspaces';
export { test as ava };
export declare type AvaWorkspaceFn = (t: ava.ExecutionContext, args: AccountArgs, workspace: WorkspaceContainerInterface) => void | Promise<void>;
export declare interface Workspace extends RawWorkspace {
    /**
     * Convenient wrapper around AVA's test function and `workspace.fork`
     * In local sandbox mode, each `workspace.test` will:
     *
     *   - start a new local blockchain
     *   - copy the state from the blockchain created in `Workspace.init`
     *   - get access to the accounts created in `Workspace.init` using the same variable names
     *   - run concurrently with all other `workspace.test` calls, keeping data isolated
     *   - shut down at the end, forgetting all new data created
     *
     * In testnet mode, the same functionality is achieved via different means,
     * since all actions must occur on one blockchain instead of N blockchains.
     *
     * `workspace.test` is added to `near-workspaces` by `near-workspaces-ava`, and is
     * shorthand for:
     *
     *     import avaTest from 'ava';
     *     import {Workspace} from 'near-workspaces';
     *     // Alternatively, you can import Workspace and ava both from near-workspaces-ava:
     *     // import {ava as avaTest, Workspace} from 'near-workspaces-ava';
     *
     *     const workspace = Workspace.init(...);
     *
     *     avaTest('some behavior', async test => {
     *       await workspace.fork(async ({root, ...}) => {
     *         ...
     *       });
     *     });
     *
     * Instead, with the syntax sugar, you can write this as you see it below –
     * saving an indentation level and avoiding one extra `await`.
     *
     * @param description title of test run by AVA, shown in test output
     * @param fn body of test; has access to `root` and other accounts returned from function passed to `Workspace.init`. Example: `workspace.fork(async ({root, alice, bob}) => {...})`
     */
    test(description: string, fn?: AvaWorkspaceFn): void;
}
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
export declare class Workspace extends RawWorkspace {
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
    static init(configOrFunction?: InitWorkspaceFn | Partial<Config>, f?: InitWorkspaceFn): Workspace;
}
//# sourceMappingURL=index.d.ts.map
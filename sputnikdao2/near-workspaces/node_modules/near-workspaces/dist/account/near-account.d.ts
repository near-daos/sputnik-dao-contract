/// <reference types="node" />
import { URL } from 'url';
import { Buffer } from 'buffer';
import BN from 'bn.js';
import { NEAR } from 'near-units';
import { KeyPair } from 'near-api-js';
import { AccountBalance, PublicKey, CodeResult, AccountView, Empty, StateItem } from '../types';
import { ContractState } from '../contract-state';
import { Transaction } from '../transaction';
import { TransactionResult } from '../transaction-result';
import { AccessKeyData, AccountData, Records } from '../record';
export interface NearAccount {
    /** Full account id for given account. */
    readonly accountId: string;
    /**
     * Returns infomation about the account.
     * @see {@link https://docs.near.org/docs/develop/front-end/rpc#view-account}
     */
    accountView(): Promise<AccountView>;
    availableBalance(): Promise<NEAR>;
    /** Current balance of account on network. */
    balance(): Promise<AccountBalance>;
    /**
     * Create a Transaction that can be used to build actions like transfer, createAccount, etc.
     * Then once built can be signed and transmitted.
     * E.g.
     * ```ts
     * const result = await account.createTransaction(bob).transfer(NEAR.parse("1N")).signAndSend();
     * ```
     * @param receiver account that the transaction is addressed to.
     */
    createTransaction(receiver: NearAccount | string): Transaction;
    /** Test whether an account exists on the network */
    exists(): Promise<boolean>;
    /**
     * Gets users key from key store.
     */
    getKey(): Promise<KeyPair | null>;
    /**
     * Adds a key pair to key store and creates a random pair if not provided
     * @param keyPair to add keystore
     */
    setKey(keyPair?: KeyPair): Promise<PublicKey>;
    /**
     * Create a subaccount from this account
     * @param accountId either prefix for new account or full accountId with current contract as suffix.
     * @param options `keyPair` is key to be added to keystore, otherwise random one will be added.
     *                `initialBalance` how much yoctoNear to transfer to new account.
     */
    createAccount(accountId: string, options?: {
        keyPair?: KeyPair;
        initialBalance?: string;
        isSubAccount?: boolean;
    }): Promise<NearAccount>;
    /**
     * Create an account, copying Wasm bytes and contract name from a given `testnetContract` or `mainnetContract`.
     *
     * This makes use of Sandbox's patch state feature, and so only works in Sandbox mode.
     *
     * You can include `withData: true` to copy account data as well, but this is
     * currently limited by the default RPC limit of 50kB. You could set up your
     * own RPC server to get around this limit (using your own RPC endpoint will
     * be easier soon).
     *
     * @param options
     */
    createAccountFrom(options: {
        testnetContract?: string;
        mainnetContract?: string;
        withData?: boolean;
        keyPair?: KeyPair;
        initialBalance?: string;
        block_id?: number | string;
        isSubAccount?: boolean;
    }): Promise<NearAccount>;
    /** Adds suffix to accountId if account isn't sub account or have full including top level account */
    getAccount(accountId: string): NearAccount;
    /** Does not attempt to make account a subaccount of current account. */
    getFullAccount(accountId: string): NearAccount;
    /**
     * Creates an account for a contract and then deploys a Wasm binary to its account.
     * If method arguments are provided a function call to `method` will be added to the transaction so that
     * the contract can be initialized in the same step.
     *
     * @param accountId Name of contract to deploy
     * @param wasm path or data of contract binary
     * @param options If any method is passed it will be added to the transaction so that contract will be initialized
     */
    createAndDeploy(accountId: string, wasm: string | URL | Uint8Array | Buffer, options?: {
        args?: Record<string, unknown> | Uint8Array;
        attachedDeposit?: string | BN;
        gas?: string | BN;
        initialBalance?: BN | string;
        keyPair?: KeyPair;
        method?: string;
        isSubAccount?: boolean;
    }): Promise<NearAccount>;
    /**
     * Call a NEAR contract and return full results with raw receipts, etc. Example:
     *
     *     await call('lol.testnet', 'set_status', { message: 'hello' }, new BN(30 * 10**12), '0')
     *
     * @returns nearAPI.providers.FinalExecutionOutcome
     */
    call_raw(contractId: NearAccount | string, methodName: string, args: Record<string, unknown>, options?: {
        gas?: string | BN;
        attachedDeposit?: string | BN;
        signWithKey?: KeyPair;
    }): Promise<TransactionResult>;
    /**
     * Convenient wrapper around lower-level `call_raw` that returns only successful result of call, or throws error encountered during call.  Example:
     *
     *     await call('lol.testnet', 'set_status', { message: 'hello' }, new BN(30 * 10**12), '0')
     *
     * @returns any parsed return value, or throws with an error if call failed
     */
    call<T>(contractId: NearAccount | string, methodName: string, args: Record<string, unknown>, options?: {
        gas?: string | BN;
        attachedDeposit?: string | BN;
        signWithKey?: KeyPair;
    }): Promise<T>;
    /**
     * Get full response from RPC about result of view method
     * @param method contract method
     * @param args args to pass to method if required
     */
    view_raw(method: string, args?: Record<string, unknown>): Promise<CodeResult>;
    /**
     * Get the parsed result returned by view method
     * @param method contract method
     * @param args args to pass to method if required
     */
    view<T>(method: string, args?: Record<string, unknown>): Promise<T>;
    /**
     * Download contract code from provider
     */
    viewCode(): Promise<Buffer>;
    /**
     * Download contract code encoded as a Base64 string
     */
    viewCodeRaw(): Promise<string>;
    /**
     * Get the data of a contract as a map of raw key/values
     * @param prefix optional prefix of key in storage. Default is ''.
     */
    viewState(prefix?: string | Uint8Array): Promise<ContractState>;
    /**
     * Get raw contract data as base64 encoded strings.
     * @param prefix optional prefix of key in storage. Default is ''.
     */
    viewStateRaw(prefix?: string | Uint8Array): Promise<StateItem[]>;
    /** Update record to sandbox */
    sandbox_patch_state(records: Records): Promise<Empty>;
    /**
     *
     * @param key key to update in storage
     * @param value_ Data to be serialized to JSON by default
     * @param borshSchema If passed will be used to encode the data
     */
    patchState(key: string, value_: any, borshSchema?: any): Promise<any>;
    /** Delete account and sends funds to beneficiaryId */
    delete(beneficiaryId: string, keyPair?: KeyPair): Promise<TransactionResult>;
    /**
     * Adds the current account's id as the root account `<accountId>.<thisAccountID>`
     * @param accountId prefix of subaccount
     */
    makeSubAccount(accountId: string): string;
    /**
     * Test whether an accountId is a subaccount of the current account.
     * @param accountId Account to test
     */
    subAccountOf(accountId: string): boolean;
    /**
     * Used to encode the account as the the accountId string when used in `JSON.stringify`
     */
    toJSON(): string;
    /**
    * Transfer yoctoNear to another account
    */
    transfer(accountId: string | NearAccount, amount: string | BN): Promise<TransactionResult>;
    /**
     * Update the account balance, storage usage, locked_amount.
     *
     * Uses sandbox_patch_state to update the account without a transaction. Only works with network: 'sandbox'.
     */
    updateAccount(accountData?: Partial<AccountData>): Promise<Empty>;
    /**
     * Add AccessKey to account.
     *
     * Uses sandbox_patch_state to update the account without a transaction. Only works with network: 'sandbox'.
     */
    updateAccessKey(key: string | PublicKey | KeyPair, access_key_data?: AccessKeyData): Promise<Empty>;
    /**
     * Deploy contract to account.
     *
     * Uses sandbox_patch_state to update the account without a transaction. Only works with network: 'sandbox'.
     */
    updateContract(binary: Buffer | string): Promise<Empty>;
    /**
     * Update contract data of account.
     *
     * Uses sandbox_patch_state to update the account without a transaction. Only works with network: 'sandbox'.
     *
     * @param data Base64 encoded string or Buffer to be encoded as Base64
     * @param value Base64 encoded string or Buffer to be encoded as Base64
     */
    updateData(data: string | Buffer, value: string | Buffer): Promise<Empty>;
}
//# sourceMappingURL=near-account.d.ts.map
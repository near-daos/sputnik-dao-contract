/// <reference types="node" />
import { URL } from 'url';
import { Buffer } from 'buffer';
import BN from 'bn.js';
import { NEAR } from 'near-units';
import { KeyPair, PublicKey, CodeResult, AccountBalance, Args, AccountView, Empty, StateItem } from '../types';
import { Transaction } from '../transaction';
import { ContractState } from '../contract-state';
import { JsonRpcProvider } from '../jsonrpc';
import { TransactionResult } from '../transaction-result';
import { AccessKeyData, AccountData, Records } from '../record';
import { NearAccount } from './near-account';
import { NearAccountManager } from './near-account-manager';
export declare class Account implements NearAccount {
    private readonly _accountId;
    private readonly manager;
    constructor(_accountId: string, manager: NearAccountManager);
    accountView(): Promise<AccountView>;
    exists(): Promise<boolean>;
    protected get provider(): JsonRpcProvider;
    get accountId(): string;
    availableBalance(): Promise<NEAR>;
    balance(): Promise<AccountBalance>;
    createTransaction(receiver: NearAccount | string): Transaction;
    getKey(): Promise<KeyPair | null>;
    setKey(keyPair?: KeyPair): Promise<PublicKey>;
    createAccount(accountId: string, { keyPair, initialBalance, isSubAccount, }?: {
        keyPair?: KeyPair;
        initialBalance?: string;
        isSubAccount?: boolean;
    }): Promise<NearAccount>;
    createAccountFrom({ testnetContract, mainnetContract, withData, block_id, keyPair, initialBalance, }: {
        testnetContract?: string;
        mainnetContract?: string;
        withData?: boolean;
        keyPair?: KeyPair;
        initialBalance?: string;
        block_id?: number | string;
    }): Promise<NearAccount>;
    getAccount(accountId: string): NearAccount;
    getFullAccount(accountId: string): NearAccount;
    createAndDeploy(accountId: string, wasm: string | URL | Uint8Array | Buffer, { attachedDeposit, args, gas, initialBalance, keyPair, method, isSubAccount, }?: {
        args?: Record<string, unknown> | Uint8Array;
        attachedDeposit?: string | BN;
        gas?: string | BN;
        initialBalance?: BN | string;
        keyPair?: KeyPair;
        method?: string;
        isSubAccount?: boolean;
    }): Promise<NearAccount>;
    call_raw(contractId: NearAccount | string, methodName: string, args: Record<string, unknown> | Uint8Array, { gas, attachedDeposit, signWithKey, }?: {
        gas?: string | BN;
        attachedDeposit?: string | BN;
        signWithKey?: KeyPair;
    }): Promise<TransactionResult>;
    call<T>(contractId: NearAccount | string, methodName: string, args: Record<string, unknown> | Uint8Array, { gas, attachedDeposit, signWithKey, }?: {
        gas?: string | BN;
        attachedDeposit?: string | BN;
        signWithKey?: KeyPair;
    }): Promise<T>;
    view_raw(method: string, args?: Args): Promise<CodeResult>;
    view<T>(method: string, args?: Args): Promise<T>;
    viewCode(): Promise<Buffer>;
    viewCodeRaw(): Promise<string>;
    viewState(prefix?: string | Uint8Array): Promise<ContractState>;
    viewStateRaw(prefix?: string | Uint8Array): Promise<StateItem[]>;
    patchState(key: string, value_: any, borshSchema?: any): Promise<Empty>;
    sandbox_patch_state(records: Records): Promise<Empty>;
    delete(beneficiaryId: string, keyPair?: KeyPair): Promise<TransactionResult>;
    makeSubAccount(accountId: string): string;
    subAccountOf(accountId: string): boolean;
    toJSON(): string;
    updateAccount(accountData?: Partial<AccountData>): Promise<Empty>;
    updateAccessKey(key: string | PublicKey | KeyPair, access_key_data?: AccessKeyData): Promise<Empty>;
    updateContract(binary: Buffer | string): Promise<Empty>;
    updateData(key: string | Buffer, value: string | Buffer): Promise<Empty>;
    transfer(accountId: string | NearAccount, amount: string | BN): Promise<TransactionResult>;
    protected internalCreateAccount(accountId: string, { keyPair, initialBalance, isSubAccount, }?: {
        keyPair?: KeyPair;
        initialBalance?: string | BN;
        isSubAccount?: boolean;
    }): Promise<Transaction>;
    private getOrCreateKey;
    private recordBuilder;
}
//# sourceMappingURL=account.d.ts.map
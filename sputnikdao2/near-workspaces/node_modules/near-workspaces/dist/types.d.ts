/// <reference types="node" />
import { ChildProcess } from 'child_process';
import _BN from 'bn.js';
import { KeyPair } from 'near-api-js';
import { Output } from 'promisify-child-process';
import { NEAR } from 'near-units';
export { ServerError } from 'near-api-js/lib/utils/rpc_errors';
export { KeyPair, Connection } from 'near-api-js';
export { PublicKey, KeyPairEd25519 } from 'near-api-js/lib/utils';
export { Action, createAccount, deployContract, functionCall, transfer, stake, addKey, deleteKey, deleteAccount, fullAccessKey, AccessKey, } from 'near-api-js/lib/transaction';
export { JsonRpcProvider as JSONRpc } from 'near-api-js/lib/providers/json-rpc-provider';
export { KeyStore } from 'near-api-js/lib/key_stores';
export * from 'near-api-js/lib/providers/provider';
export { DEFAULT_FUNCTION_CALL_GAS } from 'near-api-js/lib/constants';
export declare class BN extends _BN {
    toJSON(): string;
}
export declare type Args = Record<string, any>;
export interface NamedAccount {
    accountId: string;
}
export interface CallOptions {
    gas?: string | BN;
    attachedDeposit?: string | BN;
    signWithKey?: KeyPair;
}
export declare type ChildProcessPromise = Promise<ChildProcess & Promise<Output>>;
export interface AccountBalance {
    total: NEAR;
    stateStaked: NEAR;
    staked: NEAR;
    available: NEAR;
}
export declare type Network = 'testnet' | 'mainnet' | 'sandbox';
export interface ClientConfig {
    network: Network;
    rootAccount?: string;
    rpcAddr: string;
    helperUrl?: string;
    explorerUrl?: string;
    initialBalance?: string;
    walletUrl?: string;
    archivalUrl?: string;
}
export declare const TESTNET_RPC_ADDR = "https://archival-rpc.testnet.near.org";
export declare const MAINNET_RPC_ADDR = "https://archival-rpc.mainnet.near.org";
export interface StateItem {
    key: string;
    value: string;
    proof: string[];
}
export declare type Empty = {};
//# sourceMappingURL=types.d.ts.map
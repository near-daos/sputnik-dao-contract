/// <reference types="node" />
import { Buffer } from 'buffer';
import { Records } from './record';
import { JSONRpc, AccountView, NearProtocolConfig, AccountBalance, CodeResult, BlockId, Finality, StateItem, Empty, PublicKey, Network } from './types';
/**
 * Extends the main provider class in NAJ, adding more methods for
 * interacting with an endpoint.
 */
export declare class JsonRpcProvider extends JSONRpc {
    private static readonly providers;
    /**
     *
     * @param config rpc endpoint URL or a configuration that includes one.
     * @returns
     */
    static from(config: string | {
        rpcAddr: string;
    }): JsonRpcProvider;
    static fromNetwork(network: Network): JsonRpcProvider;
    /**
     * Download the binary of a given contract.
     * @param account_id contract account
     * @returns Buffer of Wasm binary
     */
    viewCode(account_id: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<Buffer>;
    /**
     * Download the binary of a given contract.
     * @param account_id contract account
     * @returns Base64 string of Wasm binary
     */
    viewCodeRaw(account_id: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<string>;
    viewAccount(account_id: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<AccountView>;
    accountExists(account_id: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<boolean>;
    view_access_key(account_id: string, publicKey: PublicKey | string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<any>;
    protocolConfig(blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<NearProtocolConfig>;
    account_balance(account_id: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<AccountBalance>;
    view_call(account_id: string, method_name: string, args: Record<string, unknown>, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<CodeResult>;
    /**
     *
     * @param account_id
     * @param method_name
     * @param args Base64 encoded string
     * @param blockQuery
     * @returns
     */
    view_call_raw(account_id: string, method_name: string, args_base64: string, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<CodeResult>;
    /**
     * Download the state of a contract given a prefix of a key.
     *
     * @param account_id contract account to lookup
     * @param prefix string or byte prefix of keys to loodup
     * @param blockQuery state at what block, defaults to most recent final block
     * @returns raw RPC response
     */
    viewState(account_id: string, prefix: string | Uint8Array, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<Array<{
        key: Buffer;
        value: Buffer;
    }>>;
    /**
     * Download the state of a contract given a prefix of a key without decoding from base64.
     *
     * @param account_id contract account to lookup
     * @param prefix string or byte prefix of keys to loodup
     * @param blockQuery state at what block, defaults to most recent final block
     * @returns raw RPC response
     */
    viewStateRaw(account_id: string, prefix: string | Uint8Array, blockQuery?: {
        block_id: BlockId;
    } | {
        finality: Finality;
    }): Promise<StateItem[]>;
    /**
     * Updates records without using a transaction.
     * Note: only avaialable on Sandbox endpoints.
     * @param records
     * @returns
     */
    sandbox_patch_state(records: Records): Promise<Empty>;
}
export declare const TestnetRpc: JsonRpcProvider;
export declare const MainnetRpc: JsonRpcProvider;
//# sourceMappingURL=jsonrpc.d.ts.map
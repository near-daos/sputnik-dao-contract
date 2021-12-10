"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MainnetRpc = exports.TestnetRpc = exports.JsonRpcProvider = void 0;
// eslint-disable unicorn/no-object-as-default-parameter
const buffer_1 = require("buffer");
const near_units_1 = require("near-units");
const types_1 = require("./types");
const OPTIMISTIC = { finality: 'optimistic' };
/**
 * Extends the main provider class in NAJ, adding more methods for
 * interacting with an endpoint.
 */
class JsonRpcProvider extends types_1.JSONRpc {
    /**
     *
     * @param config rpc endpoint URL or a configuration that includes one.
     * @returns
     */
    static from(config) {
        const url = typeof config === 'string' ? config : config.rpcAddr;
        if (!this.providers.has(url)) {
            this.providers.set(url, new JsonRpcProvider(url));
        }
        return this.providers.get(url);
    }
    static fromNetwork(network) {
        switch (network) {
            case 'mainnet': return exports.MainnetRpc;
            case 'testnet': return exports.TestnetRpc;
            default: throw new TypeError('Invalid network only mainnet or testnet');
        }
    }
    /**
     * Download the binary of a given contract.
     * @param account_id contract account
     * @returns Buffer of Wasm binary
     */
    async viewCode(account_id, blockQuery) {
        return buffer_1.Buffer.from(await this.viewCodeRaw(account_id, blockQuery), 'base64');
    }
    /**
     * Download the binary of a given contract.
     * @param account_id contract account
     * @returns Base64 string of Wasm binary
     */
    async viewCodeRaw(account_id, blockQuery = OPTIMISTIC) {
        const { code_base64 } = await this.query({
            request_type: 'view_code',
            account_id,
            ...blockQuery,
        });
        return code_base64;
    }
    async viewAccount(account_id, blockQuery = OPTIMISTIC) {
        return this.query({
            request_type: 'view_account',
            account_id,
            ...blockQuery,
        });
    }
    async accountExists(account_id, blockQuery) {
        try {
            await this.viewAccount(account_id, blockQuery);
            return true;
        }
        catch {
            return false;
        }
    }
    async view_access_key(account_id, publicKey, blockQuery = OPTIMISTIC) {
        return this.query({
            request_type: 'view_access_key',
            account_id,
            public_key: typeof publicKey === 'string' ? publicKey : publicKey.toString(),
            ...blockQuery,
        });
    }
    async protocolConfig(blockQuery = OPTIMISTIC) {
        // @ts-expect-error Bad type
        return this.experimental_protocolConfig(blockQuery);
    }
    async account_balance(account_id, blockQuery) {
        const config = await this.protocolConfig(blockQuery);
        const state = await this.viewAccount(account_id, blockQuery);
        const cost = config.runtime_config.storage_amount_per_byte;
        const costPerByte = near_units_1.NEAR.from(cost);
        const stateStaked = near_units_1.NEAR.from(state.storage_usage).mul(costPerByte);
        const staked = near_units_1.NEAR.from(state.locked);
        const total = near_units_1.NEAR.from(state.amount).add(staked);
        const available = total.sub(staked.max(stateStaked));
        return {
            total,
            stateStaked,
            staked,
            available,
        };
    }
    async view_call(account_id, method_name, args, blockQuery) {
        return this.view_call_raw(account_id, method_name, buffer_1.Buffer.from(JSON.stringify(args)).toString('base64'), blockQuery);
    }
    /**
     *
     * @param account_id
     * @param method_name
     * @param args Base64 encoded string
     * @param blockQuery
     * @returns
     */
    async view_call_raw(account_id, method_name, args_base64, blockQuery = OPTIMISTIC) {
        return this.query({
            request_type: 'call_function',
            account_id,
            method_name,
            args_base64,
            ...blockQuery,
        });
    }
    /**
     * Download the state of a contract given a prefix of a key.
     *
     * @param account_id contract account to lookup
     * @param prefix string or byte prefix of keys to loodup
     * @param blockQuery state at what block, defaults to most recent final block
     * @returns raw RPC response
     */
    async viewState(account_id, prefix, blockQuery) {
        const values = await this.viewStateRaw(account_id, prefix, blockQuery);
        return values.map(({ key, value }) => ({
            key: buffer_1.Buffer.from(key, 'base64'),
            value: buffer_1.Buffer.from(value, 'base64'),
        }));
    }
    /**
     * Download the state of a contract given a prefix of a key without decoding from base64.
     *
     * @param account_id contract account to lookup
     * @param prefix string or byte prefix of keys to loodup
     * @param blockQuery state at what block, defaults to most recent final block
     * @returns raw RPC response
     */
    async viewStateRaw(account_id, prefix, blockQuery) {
        const { values } = await this.query({
            request_type: 'view_state',
            ...(blockQuery !== null && blockQuery !== void 0 ? blockQuery : { finality: 'optimistic' }),
            account_id,
            prefix_base64: buffer_1.Buffer.from(prefix).toString('base64'),
        });
        return values;
    }
    /**
     * Updates records without using a transaction.
     * Note: only avaialable on Sandbox endpoints.
     * @param records
     * @returns
     */
    async sandbox_patch_state(records) {
        return this.sendJsonRpc('sandbox_patch_state', records);
    }
}
exports.JsonRpcProvider = JsonRpcProvider;
JsonRpcProvider.providers = new Map();
exports.TestnetRpc = JsonRpcProvider.from(types_1.TESTNET_RPC_ADDR);
exports.MainnetRpc = JsonRpcProvider.from(types_1.MAINNET_RPC_ADDR);
//# sourceMappingURL=jsonrpc.js.map
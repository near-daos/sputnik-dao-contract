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
exports.Account = void 0;
const buffer_1 = require("buffer");
const borsh = __importStar(require("borsh"));
const types_1 = require("../types");
const contract_state_1 = require("../contract-state");
const jsonrpc_1 = require("../jsonrpc");
const utils_1 = require("../utils");
const transaction_result_1 = require("../transaction-result");
const record_1 = require("../record");
class Account {
    constructor(_accountId, manager) {
        this._accountId = _accountId;
        this.manager = manager;
    }
    async accountView() {
        return this.manager.accountView(this._accountId);
    }
    async exists() {
        return this.provider.accountExists(this.accountId);
    }
    get provider() {
        return this.manager.provider;
    }
    get accountId() {
        return this._accountId;
    }
    async availableBalance() {
        return this.manager.availableBalance(this.accountId);
    }
    async balance() {
        return this.manager.balance(this.accountId);
    }
    createTransaction(receiver) {
        return this.manager.createTransaction(this, receiver);
    }
    async getKey() {
        return this.manager.getKey(this.accountId);
    }
    async setKey(keyPair) {
        return (await this.manager.setKey(this.accountId, keyPair)).getPublicKey();
    }
    async createAccount(accountId, { keyPair, initialBalance, isSubAccount, } = {}) {
        const tx = await this.internalCreateAccount(accountId, {
            keyPair,
            initialBalance,
            isSubAccount,
        });
        await tx.signAndSend();
        return this.getAccount(accountId);
    }
    async createAccountFrom({ testnetContract, mainnetContract, withData = false, block_id, keyPair, initialBalance, }) {
        if ((testnetContract && mainnetContract) || !(testnetContract || mainnetContract)) {
            throw new TypeError('Provide `mainnetContract` or `testnetContract` but not both.');
        }
        const network = mainnetContract ? 'mainnet' : 'testnet';
        const refContract = (mainnetContract !== null && mainnetContract !== void 0 ? mainnetContract : testnetContract);
        const rpc = jsonrpc_1.JsonRpcProvider.fromNetwork(network);
        const blockQuery = block_id ? { block_id } : undefined;
        const account = this.getFullAccount(refContract);
        // Get account view of account on reference network
        const accountView = await rpc.viewAccount(refContract, blockQuery);
        accountView.amount = initialBalance !== null && initialBalance !== void 0 ? initialBalance : accountView.amount;
        const pubKey = await account.setKey(keyPair);
        const records = account.recordBuilder()
            .account(accountView)
            .accessKey(pubKey);
        if (accountView.code_hash !== utils_1.EMPTY_CONTRACT_HASH) {
            const binary = await rpc.viewCodeRaw(refContract, blockQuery);
            records.contract(binary);
        }
        await account.sandbox_patch_state(records);
        if (!await this.provider.accountExists(refContract)) {
            await account.sandbox_patch_state(records);
            if (!await this.provider.accountExists(refContract)) {
                throw new Error(`Account ${refContract} does not exist after trying to patch into sandbox.`);
            }
        }
        if (withData) {
            const rawData = await rpc.viewStateRaw(account.accountId, '', blockQuery);
            const data = rawData.map(({ key, value }) => ({
                Data: {
                    account_id: account.accountId, data_key: key, value,
                },
            }));
            await account.sandbox_patch_state({ records: data });
        }
        return account;
    }
    getAccount(accountId) {
        const id = this.makeSubAccount(accountId);
        return this.getFullAccount(id);
    }
    getFullAccount(accountId) {
        return new Account(accountId, this.manager);
    }
    async createAndDeploy(accountId, wasm, { attachedDeposit = utils_1.NO_DEPOSIT, args = {}, gas = types_1.DEFAULT_FUNCTION_CALL_GAS, initialBalance, keyPair, method, isSubAccount, } = {}) {
        let tx = await this.internalCreateAccount(accountId, {
            keyPair,
            initialBalance,
            isSubAccount,
        });
        tx = await tx.deployContractFile(wasm);
        if (method) {
            tx.functionCall(method, args, { gas, attachedDeposit });
        }
        await tx.signAndSend();
        return this.getAccount(accountId);
    }
    async call_raw(contractId, methodName, args, { gas = types_1.DEFAULT_FUNCTION_CALL_GAS, attachedDeposit = utils_1.NO_DEPOSIT, signWithKey = undefined, } = {}) {
        return this.createTransaction(contractId)
            .functionCall(methodName, args, { gas, attachedDeposit })
            .signAndSend(signWithKey);
    }
    async call(contractId, methodName, args, { gas = types_1.DEFAULT_FUNCTION_CALL_GAS, attachedDeposit = utils_1.NO_DEPOSIT, signWithKey = undefined, } = {}) {
        const txResult = await this.call_raw(contractId, methodName, args, {
            gas,
            attachedDeposit,
            signWithKey,
        });
        if (txResult.failed) {
            throw new transaction_result_1.TransactionError(txResult);
        }
        return txResult.parseResult();
    }
    async view_raw(method, args = {}) {
        return this.provider.view_call(this.accountId, method, args);
    }
    async view(method, args = {}) {
        const result = await this.view_raw(method, args);
        if (result.result) {
            const value = buffer_1.Buffer.from(result.result).toString();
            try {
                return JSON.parse(value);
            }
            catch {
                return value;
            }
        }
        return null;
    }
    async viewCode() {
        return this.provider.viewCode(this.accountId);
    }
    async viewCodeRaw() {
        return this.provider.viewCodeRaw(this.accountId);
    }
    async viewState(prefix = '') {
        return new contract_state_1.ContractState(await this.provider.viewState(this.accountId, prefix));
    }
    async viewStateRaw(prefix = '') {
        return this.provider.viewStateRaw(this.accountId, prefix);
    }
    async patchState(key, value_, borshSchema) {
        return this.updateData(buffer_1.Buffer.from(key), buffer_1.Buffer.from(borshSchema ? borsh.serialize(borshSchema, value_) : value_));
    }
    async sandbox_patch_state(records) {
        // FIX THIS: Shouldn't need two calls to update before next RPC view call.
        await this.provider.sandbox_patch_state(records);
        return this.provider.sandbox_patch_state(records);
    }
    async delete(beneficiaryId, keyPair) {
        const result = await this.createTransaction(this)
            .deleteAccount(beneficiaryId)
            .signAndSend(keyPair);
        if (result.succeeded && await this.getKey() !== null) {
            await this.manager.deleteKey(this.accountId);
        }
        return result;
    }
    makeSubAccount(accountId) {
        if (this.subAccountOf(accountId)
            || this.manager.root.subAccountOf(accountId)) {
            return accountId;
        }
        return `${accountId}.${this.accountId}`;
    }
    subAccountOf(accountId) {
        return accountId.endsWith(`.${this.accountId}`);
    }
    toJSON() {
        return this.accountId;
    }
    async updateAccount(accountData) {
        return this.sandbox_patch_state(this.recordBuilder().account(accountData));
    }
    async updateAccessKey(key, access_key_data) {
        return this.sandbox_patch_state(this.recordBuilder().accessKey(key, access_key_data));
    }
    async updateContract(binary) {
        const accountView = await this.accountView();
        const rb = this.recordBuilder();
        rb.account(accountView);
        return this.sandbox_patch_state(rb.contract(binary));
    }
    async updateData(key, value) {
        const key_string = key instanceof buffer_1.Buffer ? key.toString('base64') : key;
        const value_string = value instanceof buffer_1.Buffer ? value.toString('base64') : value;
        return this.sandbox_patch_state(this.recordBuilder().data(key_string, value_string));
    }
    async transfer(accountId, amount) {
        return this.createTransaction(accountId).transfer(amount).signAndSend();
    }
    async internalCreateAccount(accountId, { keyPair, initialBalance, isSubAccount = true, } = {}) {
        const newAccountId = isSubAccount ? this.makeSubAccount(accountId) : accountId;
        const pubKey = (await this.getOrCreateKey(newAccountId, keyPair)).getPublicKey();
        const amount = (initialBalance !== null && initialBalance !== void 0 ? initialBalance : this.manager.initialBalance).toString();
        return this.createTransaction(newAccountId)
            .createAccount()
            .transfer(amount)
            .addKey(pubKey);
    }
    async getOrCreateKey(accountId, keyPair) {
        var _a;
        return (_a = (await this.manager.getKey(accountId))) !== null && _a !== void 0 ? _a : this.manager.setKey(accountId, keyPair);
    }
    recordBuilder() {
        return record_1.RecordBuilder.fromAccount(this);
    }
}
exports.Account = Account;
//# sourceMappingURL=account.js.map
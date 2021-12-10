"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AccountBuilder = exports.RecordBuilder = void 0;
const types_1 = require("../types");
const utils_1 = require("../utils");
class RecordBuilder {
    constructor() {
        this.records = [];
    }
    static fromAccount(accountId) {
        return new AccountBuilder(accountId);
    }
    push(record) {
        this.records.push(record);
        return this;
    }
}
exports.RecordBuilder = RecordBuilder;
const DEFAULT_ACCOUNT_DATA = {
    // 10_000 NEAR
    amount: '10000000000000000000000000000',
    locked: '0',
    // No contract hash
    code_hash: '11111111111111111111111111111111',
    storage_usage: 0,
    version: 'V1',
};
const DEFAULT_ACCESS_KEY_PERMISSION = { nonce: 0, permission: 'FullAccess' };
function isAccount(something) {
    return 'Account' in something
        && typeof something.Account.account_id === 'string';
}
function isNamedAccount(something) {
    return 'accountId' in something
        && typeof something.accountId === 'string';
}
class AccountBuilder extends RecordBuilder {
    constructor(accountOrId) {
        super();
        if (typeof accountOrId === 'string') {
            this.account_id = accountOrId;
        }
        else if (isAccount(accountOrId)) {
            this.account_id = accountOrId.Account.account_id;
            this.push(accountOrId);
        }
        else if (isNamedAccount(accountOrId)) {
            this.account_id = accountOrId.accountId;
        }
        else {
            throw new TypeError('Only `string` or `Record.Accounts` or `NamedAccount` are allowed.');
        }
    }
    accessKey(key, access_key = DEFAULT_ACCESS_KEY_PERMISSION) {
        const public_key = typeof key === 'string' ? key
            : (key instanceof types_1.PublicKey ? key.toString()
                : key.getPublicKey().toString());
        return this.push({
            AccessKey: {
                account_id: this.account_id,
                public_key,
                access_key,
            },
        });
    }
    account(accountData) {
        const account = { ...DEFAULT_ACCOUNT_DATA, ...accountData };
        return this.push({
            Account: {
                account_id: this.account_id,
                account,
            },
        });
    }
    data(data_key, value) {
        return this.push({
            Data: { account_id: this.account_id, data_key, value },
        });
    }
    contract(binary) {
        const code = typeof binary === 'string' ? binary : binary.toString('base64');
        const record = {
            Contract: {
                account_id: this.account_id,
                code,
            },
        };
        /**
         * Check conditions on ordering and hashes in records to throw error before sandbox does.
        */
        const accountRecord = this.records.find(r => 'Account' in r && r.Account.account_id === this.account_id);
        if (!accountRecord) {
            throw new Error(`Contract record with account_id: ${this.account_id} does not have a preceding Account record.`);
        }
        if ((0, utils_1.hashContract)(record.Contract.code) !== accountRecord.Account.account.code_hash) {
            throw new Error(`The hash field of the Account record with account_id: ${this.account_id} does not equal the hash of the binary in the Contract record.`);
        }
        return this.push(record);
    }
}
exports.AccountBuilder = AccountBuilder;
//# sourceMappingURL=builder.js.map
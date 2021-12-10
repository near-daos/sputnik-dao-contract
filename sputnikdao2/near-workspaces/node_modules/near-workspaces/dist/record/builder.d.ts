/// <reference types="node" />
import { Buffer } from 'buffer';
import { KeyPair, NamedAccount, PublicKey } from '../types';
import { AccessKeyData, Account, AccountData, StateRecord } from './types';
export declare class RecordBuilder {
    readonly records: StateRecord[];
    static fromAccount(accountId: string | Account | NamedAccount): AccountBuilder;
    push(record: StateRecord): this;
}
export declare class AccountBuilder extends RecordBuilder {
    readonly account_id: string;
    constructor(accountOrId: string | Account | NamedAccount);
    accessKey(key: string | PublicKey | KeyPair, access_key?: AccessKeyData): this;
    account(accountData?: Partial<AccountData>): this;
    data(data_key: string, value: string): this;
    contract(binary: Buffer | string): this;
}
//# sourceMappingURL=builder.d.ts.map
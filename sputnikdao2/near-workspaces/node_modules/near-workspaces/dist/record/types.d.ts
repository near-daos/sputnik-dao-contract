import { FunctionCallPermissionView } from '../types';
export interface KeyData {
    public_key: string;
    access_key: AccessKeyData;
}
export interface AccessKeyData {
    nonce: number;
    permission: 'FullAccess' | FunctionCallPermissionView;
}
export interface AccessKey {
    AccessKey: {
        account_id: string;
    } & KeyData;
}
export interface AccountData {
    amount: string;
    locked: string;
    code_hash: string;
    storage_usage: number;
    version: 'V1';
}
export interface Account {
    Account: {
        account_id: string;
        account: AccountData;
    };
}
export interface Contract {
    Contract: {
        account_id: string;
        /** Base64 Encoded */
        code: string;
    };
}
export interface Data {
    Data: {
        account_id: string;
        data_key: string;
        value: string;
    };
}
export declare type StateRecord = Data | Account | AccessKey | Contract;
export interface Records {
    records: StateRecord[];
}
/**
 * Unimplemented types

    /// Postponed Action Receipt.
    PostponedReceipt(Box<Receipt>),
    /// Received data from DataReceipt encoded in base64 for the given account_id and data_id.
    ReceivedData {
        account_id: AccountId,
        data_id: CryptoHash,
        #[serde(with = "option_base64_format")]
        data: Option<Vec<u8>>,
    },
    /// Delayed Receipt.
    /// The receipt was delayed because the shard was overwhelmed.
    DelayedReceipt(Box<Receipt>),
 */
//# sourceMappingURL=types.d.ts.map
import { KeyPair } from 'near-api-js';
import { NEAR } from 'near-units';
import { TransactionResult } from '../transaction-result';
import { Config } from '../interfaces';
import { JsonRpcProvider } from '../jsonrpc';
import { Transaction } from '../transaction';
import { AccountBalance, AccountView } from '../types';
import { NearAccount } from './near-account';
export interface NearAccountManager {
    readonly provider: JsonRpcProvider;
    readonly initialBalance: string;
    readonly root: NearAccount;
    accountView(accountId: string): Promise<AccountView>;
    availableBalance(account: string | NearAccount): Promise<NEAR>;
    balance(accountId: string | NearAccount): Promise<AccountBalance>;
    executeTransaction(tx: Transaction, keyPair?: KeyPair): Promise<TransactionResult>;
    addAccountCreated(account: string, sender: string): void;
    getAccount(accountId: string): NearAccount;
    getKey(accountId: string): Promise<KeyPair | null>;
    deleteAccount(accountId: string, beneficiaryId: string, keyPair?: KeyPair): Promise<TransactionResult>;
    deleteKey(accountId: string): Promise<void>;
    cleanup(): Promise<void>;
    /** Creates a KeyPair if one is not provided */
    setKey(accountId: string, keyPair?: KeyPair): Promise<KeyPair>;
    createTransaction(sender: NearAccount | string, receiver: NearAccount | string): Transaction;
    createFrom(config: Config): Promise<NearAccountManager>;
    init(): Promise<NearAccountManager>;
}
//# sourceMappingURL=near-account-manager.d.ts.map
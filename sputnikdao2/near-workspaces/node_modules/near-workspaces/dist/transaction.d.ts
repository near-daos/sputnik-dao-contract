/// <reference types="node" />
import { Buffer } from 'buffer';
import { URL } from 'url';
import { NEAR } from 'near-units';
import { TransactionResult } from './transaction-result';
import { Action, PublicKey, AccessKey, BN, KeyPair, NamedAccount } from './types';
export declare abstract class Transaction {
    readonly receiverId: string;
    readonly senderId: string;
    readonly actions: Action[];
    private accountToBeCreated;
    private _transferAmount?;
    constructor(sender: NamedAccount | string, receiver: NamedAccount | string);
    addKey(publicKey: string | PublicKey, accessKey?: AccessKey): this;
    createAccount(): this;
    deleteAccount(beneficiaryId: string): this;
    deleteKey(publicKey: string | PublicKey): this;
    /**
     * Deploy given Wasm file to the account.
     *
     * @param code path or data of contract binary. If given an absolute path (such as one created with 'path.join(__dirname, …)') will use it directly. If given a relative path such as `res/contract.wasm`, will resolve it from the project root (meaning the location of the package.json file).
     */
    deployContractFile(code: string | URL | Uint8Array | Buffer): Promise<Transaction>;
    deployContract(code: Uint8Array | Buffer): this;
    functionCall(methodName: string, args: Record<string, unknown> | Uint8Array, { gas, attachedDeposit, }?: {
        gas?: BN | string;
        attachedDeposit?: BN | string;
    }): this;
    stake(amount: BN | string, publicKey: PublicKey | string): this;
    transfer(amount: string | BN): this;
    get accountCreated(): boolean;
    get transferAmount(): NEAR;
    abstract signAndSend(keyPair?: KeyPair): Promise<TransactionResult>;
}
//# sourceMappingURL=transaction.d.ts.map
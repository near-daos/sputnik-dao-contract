/// <reference types="node" />
import { Buffer } from 'buffer';
import BN from 'bn.js';
import { NamedAccount, KeyPair, ClientConfig, KeyStore } from './types';
export declare const ONE_NEAR: BN;
export declare function toYocto(amount: string): string;
export declare function createKeyPair(): KeyPair;
export declare function tGas(x: string | number): string;
export declare function randomAccountId(prefix?: string, suffix?: string): string;
export declare function asId(id: string | NamedAccount): string;
export declare const NO_DEPOSIT: BN;
export declare function captureError(fn: () => Promise<any>): Promise<string>;
export declare function isTopLevelAccount(accountId: string): boolean;
export declare function urlConfigFromNetwork(network: string | {
    network: string;
}): ClientConfig;
/**
 *
 * @param contract Base64 encoded binary or Buffer.
 * @returns sha256 hash of contract.
 */
export declare function hashContract(contract: string | Buffer): string;
export declare const EMPTY_CONTRACT_HASH = "11111111111111111111111111111111";
/**
 *
 * @returns network to connect to. Default 'sandbox'
 */
export declare function getNetworkFromEnv(): 'sandbox' | 'testnet';
export declare function homeKeyStore(): KeyStore;
//# sourceMappingURL=utils.d.ts.map
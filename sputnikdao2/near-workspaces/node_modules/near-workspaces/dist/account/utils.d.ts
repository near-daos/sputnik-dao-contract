import { CallSite } from 'callsites';
import { KeyPair } from '../types';
export declare function findCallerFile(): [string, number];
export declare function callsites(): CallSite[];
export interface KeyFilePrivate {
    private_key: string;
}
export interface KeyFileSecret {
    secret_key: string;
}
export declare type KeyFile = KeyFilePrivate | KeyFileSecret;
export declare function getKeyFromFile(filePath: string, create?: boolean): Promise<KeyPair>;
export declare function hashPathBase64(s: string): string;
export declare function sanitize(s: string): string;
//# sourceMappingURL=utils.d.ts.map
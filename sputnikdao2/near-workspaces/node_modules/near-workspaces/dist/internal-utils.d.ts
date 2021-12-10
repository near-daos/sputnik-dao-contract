/// <reference types="node" />
import { PathLike } from 'fs';
import { spawn as _spawn } from 'child_process';
import { URL } from 'url';
import { Binary } from 'near-sandbox';
import { ChildProcessPromise } from './types';
export declare const rm: (arg1: string) => Promise<void>;
export declare const sandboxBinary: () => Promise<Binary>;
export declare function exists(d: PathLike): Promise<boolean>;
export declare function asyncSpawn(bin: string, ...args: string[]): ChildProcessPromise;
export { _spawn as spawn };
export declare function debug(...args: any[]): void;
export declare function txDebug(tx: string): void;
export declare const copyDir: (arg1: string, arg2: string) => Promise<void>;
export declare function ensureBinary(): Promise<string>;
export declare function isPathLike(something: any): something is URL | string;
/**
 * Attempts to construct an absolute path to a file given a path relative to a
 * package.json. Searches through `module.paths` (Node's resolution search
 * paths) as described in https://stackoverflow.com/a/18721515/249801, then
 * falls back to using process.cwd() if still not found. Throws an acceptable
 * user-facing error if no file found.
 */
export declare function findFile(relativePath: string): Promise<string>;
//# sourceMappingURL=internal-utils.d.ts.map
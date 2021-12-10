/// <reference types="node" />
import { URL } from "url";
export declare class Binary {
    name: string;
    installDir: string;
    urls: URL[];
    static readonly DEFAULT_INSTALL_DIR: string;
    protected constructor(name: string, url: string | URL | string[] | URL[], installDir?: string);
    /**
     *
     * @param name binary name, e.g. 'git'
     * @param path URL of where to find binary
     * @param destination Directory to put the binary
     * @returns
     */
    static create(name: string, path: string | URL | string[] | URL[], destination?: string): Promise<Binary>;
    get binPath(): string;
    download(url: URL): Promise<void>;
    install(): Promise<boolean>;
    exists(): Promise<boolean>;
    run(cliArgs?: string[], options?: {
        stdio: ("inherit" | null)[];
    }): Promise<number>;
    runAndExit(cliArgs?: string[], options?: {
        stdio: ("inherit" | null)[];
    }): Promise<void>;
    uninstall(): Promise<void>;
}

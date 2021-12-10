"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SandboxRuntime = exports.TestnetRuntime = exports.WorkspaceContainer = void 0;
const buffer_1 = require("buffer");
const path_1 = require("path");
const near_units_1 = require("near-units");
const utils_1 = require("../utils");
const account_1 = require("../account");
const jsonrpc_1 = require("../jsonrpc");
const internal_utils_1 = require("../internal-utils");
const server_1 = require("./server");
class WorkspaceContainer {
    constructor(config, accounts) {
        this.returnedAccounts = new Map();
        this.createdAccounts = {};
        this.config = config;
        this.manager = account_1.AccountManager.create(config);
        if (accounts) {
            this.createdAccounts = accounts;
        }
    }
    static async create(config, fn) {
        var _a;
        switch ((_a = config.network) !== null && _a !== void 0 ? _a : (0, utils_1.getNetworkFromEnv)()) {
            case 'testnet':
                return TestnetRuntime.create(config, fn);
            case 'sandbox':
                return SandboxRuntime.create(config, fn);
            default:
                throw new Error(`config.network = '${config.network}' invalid; ` // eslint-disable-line @typescript-eslint/restrict-template-expressions
                    + 'must be \'testnet\' or \'sandbox\' (the default). Soon \'mainnet\'');
        }
    }
    static async createAndRun(fn, config = {}) {
        const runtime = await WorkspaceContainer.create(config);
        await runtime.fork(fn);
    }
    get accounts() {
        return { root: this.manager.root, ...Object.fromEntries(Object.entries(this.createdAccounts).map(([argName, account]) => [
                argName,
                this.manager.getAccount(account.accountId),
            ])) };
    }
    get homeDir() {
        return this.config.homeDir;
    }
    get init() {
        return this.config.init;
    }
    get root() {
        return this.manager.root;
    }
    isSandbox() {
        return this.config.network === 'sandbox';
    }
    isTestnet() {
        return this.config.network === 'testnet';
    }
    async fork(fn) {
        (0, internal_utils_1.debug)('About to runtime.run with config', this.config);
        try {
            await this.beforeRun();
            await fn(this.accounts, this);
        }
        catch (error) {
            if (error instanceof Error) {
                (0, internal_utils_1.debug)(error.stack);
            }
            throw error; // Figure out better error handling
        }
        finally {
            try {
                // Do any needed teardown
                await this.afterRun();
            }
            catch (error) {
                if (error instanceof Error) {
                    (0, internal_utils_1.debug)('Failed to clean up after run');
                    (0, internal_utils_1.debug)(error);
                    throw error; // eslint-disable-line no-unsafe-finally
                }
            }
        }
    }
    async createRun(fn) {
        (0, internal_utils_1.debug)('About to runtime.createRun with config', this.config);
        try {
            await this.beforeRun();
            const accounts = await fn({ workspace: this, root: this.root });
            this.createdAccounts = { ...this.createdAccounts, ...accounts };
            return accounts;
        }
        catch (error) {
            if (error instanceof buffer_1.Buffer || typeof error === 'string') {
                (0, internal_utils_1.debug)(error);
            }
            throw error; // Figure out better error handling
        }
        finally {
            // Do any needed teardown
            await this.afterRun();
        }
    }
    async executeTransaction(fn) {
        return fn();
    }
}
exports.WorkspaceContainer = WorkspaceContainer;
class TestnetRuntime extends WorkspaceContainer {
    static async create(config, initFn) {
        // Add better error handling
        const fullConfig = { ...this.defaultConfig, initFn, ...config };
        (0, internal_utils_1.debug)('Skipping initialization function for testnet; will run before each `workspace.fork`');
        const runtime = new TestnetRuntime(fullConfig);
        await runtime.manager.init();
        return runtime;
    }
    async createFrom() {
        const runtime = new TestnetRuntime({ ...this.config, init: false, initFn: this.config.initFn }, this.createdAccounts);
        runtime.manager = await this.manager.createFrom(runtime.config);
        return runtime;
    }
    static get defaultConfig() {
        return {
            homeDir: 'ignored',
            port: 3030,
            init: true,
            rm: false,
            refDir: null,
            ...this.clientConfig,
        };
    }
    static get clientConfig() {
        return (0, utils_1.urlConfigFromNetwork)('testnet');
    }
    static get provider() {
        return jsonrpc_1.JsonRpcProvider.from(this.clientConfig);
    }
    static get baseAccountId() {
        return 'testnet';
    }
    async beforeRun() {
        if (this.config.initFn) {
            this.createdAccounts = await this.config.initFn({ workspace: this, root: this.root });
        }
    }
    async afterRun() {
        await this.manager.cleanup();
    }
}
exports.TestnetRuntime = TestnetRuntime;
class SandboxRuntime extends WorkspaceContainer {
    // Edit genesis.json to add `sandbox` as an account
    static get BASE_ACCOUNT_ID() {
        return 'test.near';
    }
    static async defaultConfig() {
        const port = await server_1.SandboxServer.nextPort();
        return {
            ...this.clientConfig,
            homeDir: server_1.SandboxServer.randomHomeDir(),
            port,
            init: true,
            rm: false,
            refDir: null,
            rpcAddr: `http://localhost:${port}`,
        };
    }
    static async create(config, fn) {
        const defaultConfig = await this.defaultConfig();
        const sandbox = new SandboxRuntime({ ...defaultConfig, ...config });
        if (fn) {
            await sandbox.createRun(fn);
        }
        return sandbox;
    }
    async createAndRun(fn, config = {}) {
        await WorkspaceContainer.createAndRun(fn, config);
    }
    async createFrom() {
        let config = await SandboxRuntime.defaultConfig();
        config = { ...this.config, ...config, init: false, refDir: this.homeDir };
        const runtime = new SandboxRuntime(config, this.createdAccounts);
        return runtime;
    }
    get baseAccountId() {
        return SandboxRuntime.BASE_ACCOUNT_ID;
    }
    static get clientConfig() {
        return {
            network: 'sandbox',
            rootAccount: SandboxRuntime.BASE_ACCOUNT_ID,
            rpcAddr: '',
            initialBalance: near_units_1.NEAR.parse('100 N').toJSON(),
        };
    }
    get provider() {
        return jsonrpc_1.JsonRpcProvider.from(this.rpcAddr);
    }
    get rpcAddr() {
        return `http://localhost:${this.config.port}`;
    }
    async beforeRun() {
        // If (!(await exists(SandboxRuntime.LINKDROP_PATH))) {
        //   debug(`Downloading testnet's linkdrop to ${SandboxRuntime.LINKDROP_PATH}`);
        //   await fs.writeFile(SandboxRuntime.LINKDROP_PATH, await TestnetRuntime.provider.viewCode('testnet'));
        // }
        this.server = await server_1.SandboxServer.init(this.config);
        await this.server.start();
        if (this.config.init) {
            await this.manager.init();
            //   Console.log(await this.manager.getKey(this.config.rootAccount!))
            // await this.root.createAndDeploy('sandbox', SandboxRuntime.LINKDROP_PATH);
            // debug('Deployed \'sandbox\' linkdrop contract');
        }
    }
    async afterRun() {
        try {
            await this.server.close();
        }
        catch (error) {
            (0, internal_utils_1.debug)('this.server.close() threw error.', JSON.stringify(error, null, 2));
        }
    }
}
exports.SandboxRuntime = SandboxRuntime;
SandboxRuntime.LINKDROP_PATH = (0, path_1.join)(__dirname, '..', '..', 'core_contracts', 'testnet-linkdrop.wasm');
//# sourceMappingURL=runtime.js.map
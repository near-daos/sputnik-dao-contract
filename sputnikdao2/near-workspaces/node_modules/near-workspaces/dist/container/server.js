"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.SandboxServer = void 0;
const buffer_1 = require("buffer");
const process_1 = __importDefault(require("process"));
const promises_1 = require("fs/promises");
const path_1 = require("path");
const http = __importStar(require("http"));
const temp_dir_1 = __importDefault(require("temp-dir"));
const portCheck = __importStar(require("node-port-check"));
const pure_uuid_1 = __importDefault(require("pure-uuid"));
const internal_utils_1 = require("../internal-utils");
const pollData = JSON.stringify({
    jsonrpc: '2.0',
    id: 'dontcare',
    method: 'block',
    params: { finality: 'final' },
});
async function pingServer(port) {
    const options = {
        hostname: '0.0.0.0',
        port,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': buffer_1.Buffer.byteLength(pollData),
        },
    };
    return new Promise(resolve => {
        const request = http.request(options, result => {
            if (result.statusCode === 200) {
                resolve(true);
            }
            else {
                (0, internal_utils_1.debug)(`Sandbox running but got non-200 response: ${JSON.stringify(result)}`);
                resolve(false);
            }
        });
        request.on('error', _ => {
            resolve(false);
        });
        // Write data to request body
        request.write(pollData);
        request.end();
    });
}
async function sandboxStarted(port, timeout = 20000) {
    const checkUntil = Date.now() + timeout + 250;
    do {
        if (await pingServer(port)) { // eslint-disable-line no-await-in-loop
            return;
        }
        await new Promise(resolve => {
            setTimeout(() => resolve(true), 250); // eslint-disable-line @typescript-eslint/no-confusing-void-expression
        });
    } while (Date.now() < checkUntil);
    throw new Error(`Sandbox Server with port: ${port} failed to start after ${timeout}ms`);
}
function initialPort() {
    return Math.max(1024, Math.floor(Math.random() * 10000));
}
class SandboxServer {
    constructor(config) {
        this.readyToDie = false;
        this.config = config;
    }
    static async nextPort() {
        this.lastPort = await portCheck.nextAvailable(this.lastPort + 1, '0.0.0.0');
        return this.lastPort;
    }
    static randomHomeDir() {
        return (0, path_1.join)(temp_dir_1.default, 'sandbox', (new pure_uuid_1.default(4).toString()));
    }
    static async init(config) {
        this.binPath = await (0, internal_utils_1.ensureBinary)();
        const server = new SandboxServer(config);
        if (server.config.refDir) {
            await (0, internal_utils_1.rm)(server.homeDir);
            await (0, internal_utils_1.copyDir)(server.config.refDir, server.config.homeDir);
        }
        if ((await (0, internal_utils_1.exists)(server.homeDir)) && server.config.init) {
            await (0, internal_utils_1.rm)(server.homeDir);
        }
        if (server.config.init) {
            const { stderr, code } = await server.spawn('init');
            if (code && code < 0) {
                (0, internal_utils_1.debug)(stderr);
                throw new Error('Failed to spawn sandbox server');
            }
        }
        return server;
    }
    get homeDir() {
        return this.config.homeDir;
    }
    get port() {
        return this.config.port;
    }
    get rpcAddr() {
        return `http://localhost:${this.port}`;
    }
    async start() {
        const args = [
            '--home',
            this.homeDir,
            'run',
            '--rpc-addr',
            this.internalRpcAddr,
        ];
        if (process_1.default.env.NEAR_WORKSPACES_DEBUG) {
            const filePath = (0, path_1.join)(this.homeDir, 'sandboxServer.log');
            (0, internal_utils_1.debug)(`near-sandbox logs writing to file: ${filePath}`);
            const fd = await (0, promises_1.open)(filePath, 'a');
            this.subprocess = (0, internal_utils_1.spawn)(SandboxServer.binPath, args, {
                env: { RUST_BACKTRACE: 'full' },
                // @ts-expect-error FileHandle not assignable to Stream | IOType
                stdio: ['ignore', 'ignore', fd],
            });
            this.subprocess.on('exit', async () => {
                await fd.close();
            });
        }
        else {
            this.subprocess = (0, internal_utils_1.spawn)(SandboxServer.binPath, args, {
                stdio: ['ignore', 'ignore', 'ignore'],
            });
        }
        this.subprocess.on('exit', () => {
            if (!this.readyToDie) {
                (0, internal_utils_1.debug)(`Server with port ${this.port}: died horribly`);
            }
        });
        await sandboxStarted(this.port);
        return this;
    }
    async close() {
        var _a;
        this.readyToDie = true;
        if (!this.subprocess.kill('SIGINT')) {
            console.error(`Failed to kill child process with PID: ${(_a = this.subprocess.pid) !== null && _a !== void 0 ? _a : 'undefined'}`);
        }
        if (this.config.rm) {
            await (0, internal_utils_1.rm)(this.homeDir);
        }
    }
    get internalRpcAddr() {
        return `0.0.0.0:${this.port}`;
    }
    async spawn(command) {
        return (0, internal_utils_1.asyncSpawn)(SandboxServer.binPath, '--home', this.homeDir, command);
    }
}
exports.SandboxServer = SandboxServer;
SandboxServer.lastPort = initialPort();
//# sourceMappingURL=server.js.map
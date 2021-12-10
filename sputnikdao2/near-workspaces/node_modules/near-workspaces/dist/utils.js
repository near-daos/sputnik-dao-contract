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
exports.homeKeyStore = exports.getNetworkFromEnv = exports.EMPTY_CONTRACT_HASH = exports.hashContract = exports.urlConfigFromNetwork = exports.isTopLevelAccount = exports.captureError = exports.NO_DEPOSIT = exports.asId = exports.randomAccountId = exports.tGas = exports.createKeyPair = exports.toYocto = exports.ONE_NEAR = void 0;
const buffer_1 = require("buffer");
const process = __importStar(require("process"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const bn_js_1 = __importDefault(require("bn.js"));
const nearAPI = __importStar(require("near-api-js"));
const js_sha256_1 = __importDefault(require("js-sha256"));
const bs58_1 = __importDefault(require("bs58"));
exports.ONE_NEAR = new bn_js_1.default('1' + '0'.repeat(24));
function toYocto(amount) {
    return nearAPI.utils.format.parseNearAmount(amount);
}
exports.toYocto = toYocto;
function createKeyPair() {
    return nearAPI.utils.KeyPairEd25519.fromRandom();
}
exports.createKeyPair = createKeyPair;
function tGas(x) {
    if (typeof x === 'string' && Number.isNaN(Number.parseInt(x, 10))) {
        throw new TypeError(`tGas expects a number or a number-like string; got: ${x}`);
    }
    return String(x) + '0'.repeat(12);
}
exports.tGas = tGas;
// Create random number with at least 7 digits by default
function randomAccountId(prefix = 'dev-', suffix = `-${(Math.floor(Math.random() * (9999999 - 1000000)) + 1000000)}`) {
    return `${prefix}${Date.now()}${suffix}`;
}
exports.randomAccountId = randomAccountId;
function asId(id) {
    return typeof id === 'string' ? id : id.accountId;
}
exports.asId = asId;
exports.NO_DEPOSIT = new bn_js_1.default('0');
async function captureError(fn) {
    try {
        await fn();
    }
    catch (error) {
        if (error instanceof Error) {
            return error.message;
        }
    }
    throw new Error('fn succeeded when expected to throw an exception');
}
exports.captureError = captureError;
function isTopLevelAccount(accountId) {
    return accountId.includes('.');
}
exports.isTopLevelAccount = isTopLevelAccount;
function configFromDomain(network) {
    return {
        network,
        rpcAddr: `https://archival-rpc.${network}.near.org`,
        walletUrl: `https://wallet.${network}.near.org`,
        helperUrl: `https://helper.${network}.near.org`,
        explorerUrl: `https://explorer.${network}.near.org`,
        archivalUrl: `https://archival-rpc.${network}.near.org`,
    };
}
function urlConfigFromNetwork(network) {
    const networkName = typeof network === 'string' ? network : network.network;
    switch (networkName) {
        case 'sandbox':
            return {
                network: 'sandbox',
                rpcAddr: 'http://localhost',
            };
        case 'testnet':
        case 'mainnet': return configFromDomain(networkName);
        default:
            throw new Error(`Got network ${networkName}, but only accept 'sandbox', 'testnet', and 'mainnet'`);
    }
}
exports.urlConfigFromNetwork = urlConfigFromNetwork;
/**
 *
 * @param contract Base64 encoded binary or Buffer.
 * @returns sha256 hash of contract.
 */
function hashContract(contract) {
    const bytes = typeof contract === 'string' ? buffer_1.Buffer.from(contract, 'base64') : contract;
    const buffer = buffer_1.Buffer.from(js_sha256_1.default.sha256(bytes), 'hex');
    return bs58_1.default.encode(buffer);
}
exports.hashContract = hashContract;
exports.EMPTY_CONTRACT_HASH = '11111111111111111111111111111111';
/**
 *
 * @returns network to connect to. Default 'sandbox'
 */
function getNetworkFromEnv() {
    const network = process.env.NEAR_RUNNER_NETWORK;
    switch (network) {
        case 'sandbox':
        case 'testnet':
            return network;
        case undefined:
            return 'sandbox';
        default:
            throw new Error(`environment variable NEAR_RUNNER_NETWORK=${network} invalid; `
                + 'use \'testnet\', \'mainnet\', or \'sandbox\' (the default)');
    }
}
exports.getNetworkFromEnv = getNetworkFromEnv;
function homeKeyStore() {
    return new nearAPI.keyStores.UnencryptedFileSystemKeyStore(path.join(os.homedir(), '.near-credentials'));
}
exports.homeKeyStore = homeKeyStore;
//# sourceMappingURL=utils.js.map
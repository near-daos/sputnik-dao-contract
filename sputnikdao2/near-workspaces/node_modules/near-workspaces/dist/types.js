"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MAINNET_RPC_ADDR = exports.TESTNET_RPC_ADDR = exports.BN = exports.DEFAULT_FUNCTION_CALL_GAS = exports.KeyStore = exports.JSONRpc = exports.AccessKey = exports.fullAccessKey = exports.deleteAccount = exports.deleteKey = exports.addKey = exports.stake = exports.transfer = exports.functionCall = exports.deployContract = exports.createAccount = exports.Action = exports.KeyPairEd25519 = exports.PublicKey = exports.Connection = exports.KeyPair = exports.ServerError = void 0;
const bn_js_1 = __importDefault(require("bn.js"));
var rpc_errors_1 = require("near-api-js/lib/utils/rpc_errors");
Object.defineProperty(exports, "ServerError", { enumerable: true, get: function () { return rpc_errors_1.ServerError; } });
var near_api_js_1 = require("near-api-js");
Object.defineProperty(exports, "KeyPair", { enumerable: true, get: function () { return near_api_js_1.KeyPair; } });
Object.defineProperty(exports, "Connection", { enumerable: true, get: function () { return near_api_js_1.Connection; } });
var utils_1 = require("near-api-js/lib/utils");
Object.defineProperty(exports, "PublicKey", { enumerable: true, get: function () { return utils_1.PublicKey; } });
Object.defineProperty(exports, "KeyPairEd25519", { enumerable: true, get: function () { return utils_1.KeyPairEd25519; } });
var transaction_1 = require("near-api-js/lib/transaction");
Object.defineProperty(exports, "Action", { enumerable: true, get: function () { return transaction_1.Action; } });
Object.defineProperty(exports, "createAccount", { enumerable: true, get: function () { return transaction_1.createAccount; } });
Object.defineProperty(exports, "deployContract", { enumerable: true, get: function () { return transaction_1.deployContract; } });
Object.defineProperty(exports, "functionCall", { enumerable: true, get: function () { return transaction_1.functionCall; } });
Object.defineProperty(exports, "transfer", { enumerable: true, get: function () { return transaction_1.transfer; } });
Object.defineProperty(exports, "stake", { enumerable: true, get: function () { return transaction_1.stake; } });
Object.defineProperty(exports, "addKey", { enumerable: true, get: function () { return transaction_1.addKey; } });
Object.defineProperty(exports, "deleteKey", { enumerable: true, get: function () { return transaction_1.deleteKey; } });
Object.defineProperty(exports, "deleteAccount", { enumerable: true, get: function () { return transaction_1.deleteAccount; } });
Object.defineProperty(exports, "fullAccessKey", { enumerable: true, get: function () { return transaction_1.fullAccessKey; } });
Object.defineProperty(exports, "AccessKey", { enumerable: true, get: function () { return transaction_1.AccessKey; } });
var json_rpc_provider_1 = require("near-api-js/lib/providers/json-rpc-provider");
Object.defineProperty(exports, "JSONRpc", { enumerable: true, get: function () { return json_rpc_provider_1.JsonRpcProvider; } });
var key_stores_1 = require("near-api-js/lib/key_stores");
Object.defineProperty(exports, "KeyStore", { enumerable: true, get: function () { return key_stores_1.KeyStore; } });
__exportStar(require("near-api-js/lib/providers/provider"), exports);
var constants_1 = require("near-api-js/lib/constants");
Object.defineProperty(exports, "DEFAULT_FUNCTION_CALL_GAS", { enumerable: true, get: function () { return constants_1.DEFAULT_FUNCTION_CALL_GAS; } });
class BN extends bn_js_1.default {
    toJSON() {
        return this.toString(10);
    }
}
exports.BN = BN;
exports.TESTNET_RPC_ADDR = 'https://archival-rpc.testnet.near.org';
exports.MAINNET_RPC_ADDR = 'https://archival-rpc.mainnet.near.org';
//# sourceMappingURL=types.js.map
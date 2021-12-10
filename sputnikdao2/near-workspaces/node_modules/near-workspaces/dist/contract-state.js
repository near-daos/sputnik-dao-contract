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
Object.defineProperty(exports, "__esModule", { value: true });
exports.ContractState = void 0;
const buffer_1 = require("buffer");
const borsh = __importStar(require("borsh"));
class ContractState {
    constructor(dataArray) {
        this.data = new Map();
        for (const { key, value } of dataArray) {
            this.data.set(key.toString(), value);
        }
    }
    get_raw(key) {
        var _a;
        return (_a = this.data.get(key)) !== null && _a !== void 0 ? _a : buffer_1.Buffer.from('');
    }
    get(key, borshSchema) {
        const value = this.get_raw(key);
        if (borshSchema) {
            return borsh.deserialize(borshSchema.schema, borshSchema.type, value);
        }
        return value.toJSON();
    }
}
exports.ContractState = ContractState;
//# sourceMappingURL=contract-state.js.map
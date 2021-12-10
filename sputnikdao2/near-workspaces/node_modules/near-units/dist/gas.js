"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Gas = void 0;
const bn_js_1 = __importDefault(require("bn.js"));
const bn_1 = require("./bn");
const utils_1 = require("./utils");
class Gas extends bn_1.BNWrapper {
    /**
     * Converts a BN, number, or string in gas units to Gas.
     *
     * @example
     * ```ts
     * const gas  = Gas.from(new BN("10000000"))
     * const gas2 = Gas.from('1 TGas')
     * ```
     */
    static from(bn) {
        if (bn instanceof bn_js_1.default) {
            const gas = new Gas(0);
            // @ts-expect-error internal method
            bn.copy(gas); // eslint-disable-line @typescript-eslint/no-unsafe-call
            return gas;
        }
        return new Gas(bn);
    }
    /**
     * Convert human readable gas amount to internal indivisible units.
     *
     * @example
     * ```ts
     * Gas.parse('1') // => Gas<'1'> (1 gas)
     * Gas.parse('1 Tgas') // => Gas<'1000000000000'> (1e12 gas)
     * Gas.parse('1 Ggas') // => Gas<'1000000000'> (1e9 gas)
     * ```
     *
     * @param x decimal string denominated in gas, Tgas, Ggas, etc.
     * @returns new Gas object wrapping the parsed amount
     */
    static parse(x) {
        x = x.replace(utils_1.gasPattern, '').trim(); // Clean string for use with generic `parse`
        return new Gas((0, utils_1.parse)(x));
    }
    /**
     * Convert to string such as "53 Tgas" or "900 Ggas"
     * @returns string showing gas amount in a human-readable way
     */
    toHuman() {
        return (0, utils_1.toHuman)(this, 'gas', 12, 12);
    }
    from(bn) {
        return Gas.from(bn);
    }
}
exports.Gas = Gas;
//# sourceMappingURL=gas.js.map
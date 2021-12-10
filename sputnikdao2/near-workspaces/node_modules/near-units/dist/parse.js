"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parse = void 0;
const gas_1 = require("./gas");
const near_1 = require("./near");
const utils_1 = require("./utils");
/**
 * Parse a well-formatted string into a NEAR object or a Gas object.
 *
 * @example
 * ```
 * parse('1 N') // => NEAR<'1000000000000000000000000'>
 * parse('1mN') // => NEAR<'1000000000000000000000'>
 * parse('1Tgas') // => Gas<'1000000000000'>
 * ```
 * @param x string representing a value
 *
 */
function parse(x) {
    if (utils_1.gasPattern.test(x)) {
        return gas_1.Gas.parse(x.replace(utils_1.gasPattern, ''));
    }
    if (utils_1.nearPattern.test(x)) {
        return near_1.NEAR.parse(x.replace(utils_1.nearPattern, ''));
    }
    throw new Error(`Cannot parse ${x}; expected a NEAR-like string ('1N') or a gas-like string ('1Tgas')`);
}
exports.parse = parse;
//# sourceMappingURL=parse.js.map
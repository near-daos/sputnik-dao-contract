"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.toHuman = exports.getMagnitude = exports.clean = exports.parse = exports.nearPattern = exports.gasPattern = void 0;
/**
 * Internal utilities.
 *
 * - not exported from index.ts
 * - do not make sense on their own, but only as internal tools to be used from NEAR or Gas classes
 */
const bn_js_1 = __importDefault(require("bn.js"));
exports.gasPattern = /gas\s*$/i;
exports.nearPattern = /n(ear)?\s*$/i;
const possibleMetricPrefix = /([μa-z]+)$/i;
/**
 * Internal function to be used by {@link Gas.parse} and {@link NEAR.parse}
 * after they have already stripped out 'N' or 'gas'.
 *
 * @example
 * ```
 * parse('1 m', 24) // => input passed from NEAR
 * parse('1T') // => input passed from Gas
 * ```
 *
 * @param x string with number and possibly a trailing metric prefix
 * @returns string suitable for initializing a BN
 */
function parse(x, magnitude = 0) {
    var _a;
    if (!x) {
        throw new TypeError(`invalid input string: '${x.toString()}'`);
    }
    let amount = x; // Mutable copy that gets updated throughout function
    const metricPrefix = (_a = possibleMetricPrefix.exec(x)) === null || _a === void 0 ? void 0 : _a[1];
    if (metricPrefix) {
        magnitude += getMagnitude(metricPrefix);
        amount = amount.replace(possibleMetricPrefix, '');
    }
    amount = clean(amount);
    const split = amount.split('.');
    if (magnitude === 0 && split.length > 1) {
        throw new Error(`Cannot parse '${amount}'; unexpected decimal point${metricPrefix ? ` with metric prefix ${metricPrefix}` : ''}`);
    }
    if (split.length > 2) {
        throw new Error(`Cannot parse '${amount}'; too many decimal points (\`.\`)`);
    }
    const wholePart = split[0];
    const fracPart = split[1] || '';
    if (fracPart.length > magnitude) {
        throw new Error(`Cannot parse '${x}'; fractional part contains more than ${magnitude} digits`);
    }
    return `${wholePart}${fracPart.padEnd(magnitude, '0')}`;
}
exports.parse = parse;
/**
 * Removes commas, underscores, leading/trailing whitespace, and leading zeroes from the input
 * @param x A value or amount that may contain commas or underscores
 * @returns string The cleaned value
 */
function clean(x) {
    return x.trim().replace(/[,_]/g, '').replace(/^0+\b/, '');
}
exports.clean = clean;
/**
 * Get the order of magnitude of a given metric prefix. Throws an error if given string does not match a known metric prefix.
 * @param prefix string like 'c' or 'centi'
 * @returns corresponding order of magnitude (also sometimes called 'magnitude' of this metric prefix)
 */
function getMagnitude(prefix) {
    for (const [pattern, magnitude] of prefixToMagnitude.entries()) {
        if (pattern.test(prefix)) {
            return magnitude;
        }
    }
    throw new Error(`Unknown metric prefix: ${prefix}`);
}
exports.getMagnitude = getMagnitude;
const prefixToMagnitude = new Map([
    [/\bY\b/, 24],
    [/yotta/i, 24],
    [/\bZ\b/, 21],
    [/zetta/i, 21],
    [/\bE\b/, 18],
    [/exa/i, 18],
    [/\bP\b/, 15],
    [/peta/i, 15],
    [/\bT\b/, 12],
    [/tera/i, 12],
    [/\bG\b/, 9],
    [/giga/i, 9],
    [/\bM\b/, 6],
    [/mega/i, 6],
    [/\bk\b/, 3],
    [/kilo/i, 3],
    [/\bh\b/, 2],
    [/hecto/i, 2],
    [/\bda\b/, 1],
    [/deka/i, 1],
    [/\bd\b/, -1],
    [/deci/i, -1],
    [/\bc\b/, -2],
    [/centi/i, -2],
    [/\bm\b/, -3],
    [/milli/i, -3],
    [/μ/, -6],
    [/micro/i, -6],
    [/\bn\b/, -9],
    [/nano/i, -9],
    [/\bp\b/, -12],
    [/pico/i, -12],
    [/\bf\b/, -15],
    [/femto/i, -15],
    [/\ba\b/, -18],
    [/atto/i, -18],
    [/\bz\b/, -21],
    [/zepto/i, -21],
    [/\by\b/, -24],
    [/yocto/i, -24],
]);
const magnitudeToPrefix = new Map([
    [24, 'Y'],
    [21, 'Z'],
    [18, 'E'],
    [15, 'P'],
    [12, 'T'],
    [9, 'G'],
    [6, 'M'],
    [3, 'k'],
    [0, ''],
    [-3, 'm'],
    [-6, 'μ'],
    [-9, 'n'],
    [-12, 'p'],
    [-15, 'f'],
    [-18, 'a'],
    [-21, 'z'],
    [-24, 'y'],
]);
/**
 * Generic `toHuman` function used by both NEAR and Gas.
 *
 * @param x BN to convert to human-readable format
 * @param baseUnit String like 'N' or 'gas' that will be added to the end of the returned string
 * @param magnitude How many numbers go after the decimal point for "one" of these things (for NEAR this is 24; for gas it's 0)
 * @param adjustMagnitude DO NOT USE! Only used internally by this function when it calls itself recursively.
 * @returns human-readable representation of `x`
 */
function toHuman(x, baseUnit, magnitude, adjustMagnitude = 0) {
    const nomination = new bn_js_1.default(10).pow(new bn_js_1.default(magnitude));
    const quotient = x.div(nomination);
    const remainder = x.mod(nomination);
    if (quotient.gt(new bn_js_1.default(0))) {
        // Format the part before the decimal in en-US format (like "1,000");
        const integer = new Intl.NumberFormat('en-US').format(BigInt(quotient.toString(10)));
        // Leave the part after the decimal as-is (like ".00100200003")
        const fraction = remainder.eq(new bn_js_1.default(0))
            ? ''
            : `.${remainder
                .toString(10)
                .padStart(magnitude, '0')
                .replace(/0+$\b/, '')}`;
        const prefix = magnitudeToPrefix.get(adjustMagnitude);
        return `${integer}${fraction} ${prefix}${baseUnit}`;
    }
    return toHuman(x, baseUnit, magnitude - 3, adjustMagnitude - 3);
}
exports.toHuman = toHuman;
//# sourceMappingURL=utils.js.map
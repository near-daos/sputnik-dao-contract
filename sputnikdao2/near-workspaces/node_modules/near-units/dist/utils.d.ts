/**
 * Internal utilities.
 *
 * - not exported from index.ts
 * - do not make sense on their own, but only as internal tools to be used from NEAR or Gas classes
 */
import BN from 'bn.js';
export declare const gasPattern: RegExp;
export declare const nearPattern: RegExp;
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
export declare function parse(x: string, magnitude?: number): string;
/**
 * Removes commas, underscores, leading/trailing whitespace, and leading zeroes from the input
 * @param x A value or amount that may contain commas or underscores
 * @returns string The cleaned value
 */
export declare function clean(x: string): string;
/**
 * Get the order of magnitude of a given metric prefix. Throws an error if given string does not match a known metric prefix.
 * @param prefix string like 'c' or 'centi'
 * @returns corresponding order of magnitude (also sometimes called 'magnitude' of this metric prefix)
 */
export declare function getMagnitude(prefix: string): number;
/**
 * Generic `toHuman` function used by both NEAR and Gas.
 *
 * @param x BN to convert to human-readable format
 * @param baseUnit String like 'N' or 'gas' that will be added to the end of the returned string
 * @param magnitude How many numbers go after the decimal point for "one" of these things (for NEAR this is 24; for gas it's 0)
 * @param adjustMagnitude DO NOT USE! Only used internally by this function when it calls itself recursively.
 * @returns human-readable representation of `x`
 */
export declare function toHuman(x: BN, baseUnit: string, magnitude: number, adjustMagnitude?: number): string;

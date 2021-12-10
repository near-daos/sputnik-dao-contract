import { Gas } from './gas';
import { NEAR } from './near';
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
export declare function parse(x: string): Gas | NEAR;

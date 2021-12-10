declare let showOutput: boolean;
declare const output: (...input: any[]) => void;
/**
 * Get a number from within a range
 * @param {number} min
 * @param {number} max
 * @returns {number}
 */
declare const randomFromRange: (min?: number, max?: number) => number;
/**
 * Get a range of unique numbers
 * @param {number} howMany
 * @param {number[]} notIn
 * @returns {number[]}
 */
declare const getUniqueNumbers: (howMany?: number, notIn?: number[]) => number[];
/**
 * Returns the current port if available or the next one available by incrementing the port
 * @param {number} port
 * @param {string} host
 * @returns {Promise<number>}
 */
declare const nextAvailable: (port?: number, host?: string) => Promise<number>;
/**
 * Get a number of guaranteed free ports available for a host
 * @param {number} howMany
 * @param {string} host
 * @param {number[]} freePorts
 * @returns {Promise<number[]>}
 */
declare const getFreePorts: (howMany?: number, host?: string, freePorts?: number[]) => Promise<number[]>;
/**
 * Check if a port is free on a certain host
 * @param {number} port
 * @param {string} host
 * @returns {Promise<[number , string , boolean]>}
 */
declare const isFreePort: (port?: number, host?: string) => Promise<[number, string, boolean]>;

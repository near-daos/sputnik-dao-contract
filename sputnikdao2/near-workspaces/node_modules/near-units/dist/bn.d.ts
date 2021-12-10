import BN from 'bn.js';
export declare abstract class BNWrapper<T extends BN> extends BN {
    /**
     * @description returns the maximum of 2 BN instances.
     */
    max(other: BN): T;
    /**
     * @description returns the minimum of 2 BN instances.
     */
    min(other: BN): T;
    /**
     * @description  clone number
     */
    clone(): T;
    /**
     * @description convert to two's complement representation, where width is bit width
     */
    toTwos(width: number): T;
    /**
     * @description  convert from two's complement representation, where width is the bit width
     */
    fromTwos(width: number): T;
    /**
     * @description negate sign
     */
    neg(): T;
    /**
     * @description negate sign
     */
    ineg(): T;
    /**
     * @description absolute value
     */
    abs(): T;
    /**
     * @description absolute value
     */
    iabs(): T;
    /**
     * @description addition
     */
    add(b: BN): T;
    /**
     * @description  addition
     */
    iadd(b: BN): T;
    /**
     * @description addition
     */
    addn(b: number): T;
    /**
     * @description addition
     */
    iaddn(b: number): T;
    /**
     * @description subtraction
     */
    sub(b: BN): T;
    /**
     * @description subtraction
     */
    isub(b: BN): T;
    /**
     * @description subtraction
     */
    subn(b: number): T;
    /**
     * @description subtraction
     */
    isubn(b: number): T;
    /**
     * @description multiply
     */
    mul(b: BN): T;
    /**
     * @description multiply
     */
    imul(b: BN): T;
    /**
     * @description multiply
     */
    muln(b: number): T;
    /**
     * @description multiply
     */
    imuln(b: number): T;
    /**
     * @description square
     */
    sqr(): T;
    /**
     * @description square
     */
    isqr(): T;
    /**
     * @description raise `a` to the power of `b`
     */
    pow(b: BN): T;
    /**
     * @description divide
     */
    div(b: BN): T;
    /**
     * @description divide
     */
    divn(b: number): T;
    /**
     * @description divide
     */
    idivn(b: number): T;
    /**
     * @description reduct
     */
    mod(b: BN): T;
    /**
     * @description reduct
     */
    umod(b: BN): T;
    /**
     * @description  rounded division
     */
    divRound(b: BN): T;
    /**
     * @description or
     */
    or(b: BN): T;
    /**
     * @description or
     */
    ior(b: BN): T;
    /**
     * @description or
     */
    uor(b: BN): T;
    /**
     * @description or
     */
    iuor(b: BN): T;
    /**
     * @description and
     */
    and(b: BN): T;
    /**
     * @description and
     */
    iand(b: BN): T;
    /**
     * @description and
     */
    uand(b: BN): T;
    /**
     * @description and
     */
    iuand(b: BN): T;
    /**
     * @description and (NOTE: `andln` is going to be replaced with `andn` in future)
     */
    andln(b: number): T;
    /**
     * @description xor
     */
    xor(b: BN): T;
    /**
     * @description xor
     */
    ixor(b: BN): T;
    /**
     * @description xor
     */
    uxor(b: BN): T;
    /**
     * @description xor
     */
    iuxor(b: BN): T;
    /**
     * @description set specified bit to 1
     */
    setn(b: number): T;
    /**
     * @description shift left
     */
    shln(b: number): T;
    /**
     * @description shift left
     */
    ishln(b: number): T;
    /**
     * @description shift left
     */
    ushln(b: number): T;
    /**
     * @description shift left
     */
    iushln(b: number): T;
    /**
     * @description shift right
     */
    shrn(b: number): T;
    /**
     * @description shift right (unimplemented https://github.com/indutny/bn.js/blob/master/lib/bn.js#L2086)
     */
    ishrn(b: number): T;
    /**
     * @description shift right
     */
    ushrn(b: number): T;
    /**
     * @description shift right
     */
    iushrn(b: number): T;
    /**
     * @description  test if specified bit is set
     */
    maskn(b: number): T;
    /**
     * @description clear bits with indexes higher or equal to `b`
     */
    imaskn(b: number): T;
    /**
     * @description add `1 << b` to the number
     */
    bincn(b: number): T;
    /**
     * @description not (for the width specified by `w`)
     */
    notn(w: number): T;
    /**
     * @description not (for the width specified by `w`)
     */
    inotn(w: number): T;
    /**
     * @description GCD
     */
    gcd(b: BN): T;
    /**
     * @description Extended GCD results `({ a: ..., b: ..., gcd: ... })`
     */
    egcd(bn: BN): {
        a: T;
        b: T;
        gcd: T;
    };
    /**
     * @description inverse `a` modulo `b`
     */
    invm(b: BN): T;
    /**
     * Convert to BigInt type
     * @returns BigInt
     */
    toBigInt(): bigint;
    toJSON(): string;
    toString(base?: number | 'hex', length?: number): string;
    protected abstract from(bn: BN | number | string): T;
}

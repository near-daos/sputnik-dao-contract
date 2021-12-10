"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.BNWrapper = void 0;
const bn_js_1 = __importDefault(require("bn.js"));
class BNWrapper extends bn_js_1.default {
    /**
     * @description returns the maximum of 2 BN instances.
     */
    max(other) {
        return this.from(bn_js_1.default.max(this, other));
    }
    /**
     * @description returns the minimum of 2 BN instances.
     */
    min(other) {
        return this.from(bn_js_1.default.min(this, other));
    }
    /**
     * @description  clone number
     */
    clone() {
        return this.from(super.clone());
    }
    /**
     * @description convert to two's complement representation, where width is bit width
     */
    toTwos(width) {
        return this.from(super.toTwos(width));
    }
    /**
     * @description  convert from two's complement representation, where width is the bit width
     */
    fromTwos(width) {
        return this.from(super.fromTwos(width));
    }
    /**
     * @description negate sign
     */
    neg() {
        return this.from(super.neg());
    }
    /**
     * @description negate sign
     */
    ineg() {
        return this.from(super.ineg());
    }
    /**
     * @description absolute value
     */
    abs() {
        return this.from(super.abs());
    }
    /**
     * @description absolute value
     */
    iabs() {
        return this.from(super.iabs());
    }
    /**
     * @description addition
     */
    add(b) {
        return this.from(super.add(b));
    }
    /**
     * @description  addition
     */
    iadd(b) {
        return this.from(super.iadd(b));
    }
    /**
     * @description addition
     */
    addn(b) {
        return this.from(super.addn(b));
    }
    /**
     * @description addition
     */
    iaddn(b) {
        return this.from(super.iaddn(b));
    }
    /**
     * @description subtraction
     */
    sub(b) {
        return this.from(super.sub(b));
    }
    /**
     * @description subtraction
     */
    isub(b) {
        return this.from(super.isub(b));
    }
    /**
     * @description subtraction
     */
    subn(b) {
        return this.from(super.subn(b));
    }
    /**
     * @description subtraction
     */
    isubn(b) {
        return this.from(super.isubn(b));
    }
    /**
     * @description multiply
     */
    mul(b) {
        return this.from(super.mul(b));
    }
    /**
     * @description multiply
     */
    imul(b) {
        return this.from(super.imul(b));
    }
    /**
     * @description multiply
     */
    muln(b) {
        return this.from(super.muln(b));
    }
    /**
     * @description multiply
     */
    imuln(b) {
        return this.from(super.imuln(b));
    }
    /**
     * @description square
     */
    sqr() {
        return this.from(super.sqr());
    }
    /**
     * @description square
     */
    isqr() {
        return this.from(super.isqr());
    }
    /**
     * @description raise `a` to the power of `b`
     */
    pow(b) {
        return this.from(super.pow(b));
    }
    /**
     * @description divide
     */
    div(b) {
        return this.from(super.div(b));
    }
    /**
     * @description divide
     */
    divn(b) {
        return this.from(super.divn(b));
    }
    /**
     * @description divide
     */
    idivn(b) {
        return this.from(super.idivn(b));
    }
    /**
     * @description reduct
     */
    mod(b) {
        return this.from(super.mod(b));
    }
    /**
     * @description reduct
     */
    umod(b) {
        return this.from(super.umod(b));
    }
    /**
     * @description  rounded division
     */
    divRound(b) {
        return this.from(super.divRound(b));
    }
    /**
     * @description or
     */
    or(b) {
        return this.from(super.or(b));
    }
    /**
     * @description or
     */
    ior(b) {
        return this.from(super.ior(b));
    }
    /**
     * @description or
     */
    uor(b) {
        return this.from(super.uor(b));
    }
    /**
     * @description or
     */
    iuor(b) {
        return this.from(super.iuor(b));
    }
    /**
     * @description and
     */
    and(b) {
        return this.from(super.and(b));
    }
    /**
     * @description and
     */
    iand(b) {
        return this.from(super.iand(b));
    }
    /**
     * @description and
     */
    uand(b) {
        return this.from(super.uand(b));
    }
    /**
     * @description and
     */
    iuand(b) {
        return this.from(super.iuand(b));
    }
    /**
     * @description and (NOTE: `andln` is going to be replaced with `andn` in future)
     */
    andln(b) {
        return this.from(super.andln(b));
    }
    /**
     * @description xor
     */
    xor(b) {
        return this.from(super.xor(b));
    }
    /**
     * @description xor
     */
    ixor(b) {
        return this.from(super.ixor(b));
    }
    /**
     * @description xor
     */
    uxor(b) {
        return this.from(super.uxor(b));
    }
    /**
     * @description xor
     */
    iuxor(b) {
        return this.from(super.iuxor(b));
    }
    /**
     * @description set specified bit to 1
     */
    setn(b) {
        return this.from(super.setn(b));
    }
    /**
     * @description shift left
     */
    shln(b) {
        return this.from(super.shln(b));
    }
    /**
     * @description shift left
     */
    ishln(b) {
        return this.from(super.ishln(b));
    }
    /**
     * @description shift left
     */
    ushln(b) {
        return this.from(super.ushln(b));
    }
    /**
     * @description shift left
     */
    iushln(b) {
        return this.from(super.iushln(b));
    }
    /**
     * @description shift right
     */
    shrn(b) {
        return this.from(super.shrn(b));
    }
    /**
     * @description shift right (unimplemented https://github.com/indutny/bn.js/blob/master/lib/bn.js#L2086)
     */
    ishrn(b) {
        return this.from(super.ishrn(b));
    }
    /**
     * @description shift right
     */
    ushrn(b) {
        return this.from(super.ushrn(b));
    }
    /**
     * @description shift right
     */
    iushrn(b) {
        return this.from(super.iushrn(b));
    }
    /**
     * @description  test if specified bit is set
     */
    maskn(b) {
        return this.from(super.maskn(b));
    }
    /**
     * @description clear bits with indexes higher or equal to `b`
     */
    imaskn(b) {
        return this.from(super.imaskn(b));
    }
    /**
     * @description add `1 << b` to the number
     */
    bincn(b) {
        return this.from(super.bincn(b));
    }
    /**
     * @description not (for the width specified by `w`)
     */
    notn(w) {
        return this.from(super.notn(w));
    }
    /**
     * @description not (for the width specified by `w`)
     */
    inotn(w) {
        return this.from(super.inotn(w));
    }
    /**
     * @description GCD
     */
    gcd(b) {
        return this.from(super.gcd(b));
    }
    /**
     * @description Extended GCD results `({ a: ..., b: ..., gcd: ... })`
     */
    egcd(bn) {
        const { a, b, gcd } = super.egcd(bn);
        return { a: this.from(a), b: this.from(b), gcd: this.from(gcd) };
    }
    /**
     * @description inverse `a` modulo `b`
     */
    invm(b) {
        return this.from(super.invm(b));
    }
    /**
     * Convert to BigInt type
     * @returns BigInt
     */
    toBigInt() {
        return BigInt(this.toString());
    }
    toJSON() {
        return this.toString();
    }
    toString(base = 10, length) {
        return super.toString(base, length);
    }
}
exports.BNWrapper = BNWrapper;
//# sourceMappingURL=bn.js.map
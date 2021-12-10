"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransactionError = exports.TransactionResult = exports.PromiseOutcome = void 0;
const buffer_1 = require("buffer");
const near_units_1 = require("near-units");
function includes(pattern) {
    if (typeof pattern === 'string') {
        return s => s.includes(pattern);
    }
    return s => pattern.test(s);
}
function parseValue(value) {
    const buffer = buffer_1.Buffer.from(value, 'base64').toString();
    try {
        return JSON.parse(buffer);
    }
    catch {
        return buffer;
    }
}
class PromiseOutcome {
    constructor(outcome) {
        this.outcome = outcome;
    }
    get errors() {
        return [];
    }
    get status() {
        return this.outcome.status;
    }
    get succeeded() {
        if (typeof this.status === 'string') {
            return false;
        }
        return this.status.SuccessValue !== undefined;
    }
    get isFailure() {
        if (typeof this.status === 'string') {
            return false;
        }
        return this.status.Failure !== undefined;
    }
    get executionStatus() {
        return this.status;
    }
    parseResult() {
        if (this.succeeded) {
            return parseValue(this.SuccessValue);
        }
        throw new Error(JSON.stringify(this.status));
    }
    get SuccessValue() {
        if (this.succeeded) {
            return this.executionStatus.SuccessValue;
        }
        return undefined;
    }
    get executionError() {
        if (this.isFailure) {
            return this.executionStatus.Failure;
        }
        return undefined;
    }
    get errorMessage() {
        var _a;
        return (_a = this.executionError) === null || _a === void 0 ? void 0 : _a.error_message;
    }
    get errorType() {
        var _a;
        return (_a = this.executionError) === null || _a === void 0 ? void 0 : _a.error_type;
    }
    get logs() {
        return this.outcome.logs;
    }
    get gas_burnt() {
        return near_units_1.Gas.from(this.outcome.gas_burnt);
    }
}
exports.PromiseOutcome = PromiseOutcome;
class TransactionResult {
    constructor(result, startMs, endMs, config) {
        this.result = result;
        this.startMs = startMs;
        this.endMs = endMs;
        this.config = config;
    }
    get durationMs() {
        return this.endMs - this.startMs;
    }
    get outcomesWithId() {
        const { result } = this;
        return [result.transaction_outcome, ...result.receipts_outcome];
    }
    get receipts_outcomes() {
        return this.result.receipts_outcome.flatMap(o => new PromiseOutcome(o.outcome));
    }
    get outcome() {
        return this.outcomesWithId.flatMap(o => o.outcome);
    }
    get outcomes() {
        return this.outcomesWithId.flatMap(o => o.outcome);
    }
    get logs() {
        return this.outcomes.flatMap(it => it.logs);
    }
    get transactionReceipt() {
        return this.result.transaction;
    }
    get errors() {
        const errors = [...this.promiseErrors];
        if (this.Failure) {
            errors.unshift(this.Failure);
        }
        return errors;
    }
    get status() {
        return this.result.status;
    }
    get succeeded() {
        if (typeof this.result.status === 'string') {
            return false;
        }
        return this.result.status.SuccessValue !== undefined;
    }
    get SuccessValue() {
        if (this.succeeded) {
            return this.finalExecutionStatus.SuccessValue;
        }
        return null;
    }
    get failed() {
        if (typeof this.result.status === 'string') {
            return false;
        }
        return this.result.status.Failure !== undefined;
    }
    get Failure() {
        if (this.failed) {
            return this.finalExecutionStatus.Failure;
        }
        return null;
    }
    logsContain(pattern) {
        return this.logs.some(includes(pattern));
    }
    findLogs(pattern) {
        return this.logs.filter(includes(pattern));
    }
    promiseValuesContain(pattern) {
        return this.promiseSuccessValues.some(includes(pattern));
    }
    findPromiseValues(pattern) {
        return this.promiseSuccessValues.filter(includes(pattern));
    }
    get finalExecutionStatus() {
        return this.status;
    }
    get promiseErrors() {
        return this.receipts_outcomes.flatMap(o => { var _a; return (_a = o.executionError) !== null && _a !== void 0 ? _a : []; });
    }
    get promiseSuccessValues() {
        return this.receipts_outcomes.flatMap(o => { var _a; return (_a = o.SuccessValue) !== null && _a !== void 0 ? _a : []; });
    }
    get promiseErrorMessages() {
        return this.promiseErrors.map(error => JSON.stringify(error));
    }
    get gas_burnt() {
        return near_units_1.Gas.from(this.result.transaction_outcome.outcome.gas_burnt);
    }
    promiseErrorMessagesContain(pattern) {
        return this.promiseErrorMessages.some(includes(pattern));
    }
    parseResult() {
        if (this.succeeded) {
            return parseValue(this.SuccessValue);
        }
        throw new Error(JSON.stringify(this.status));
    }
    parsedPromiseResults() {
        return this.promiseSuccessValues.map(parseValue);
    }
    summary() {
        return `(${this.durationMs} ms) burned ${this.gas_burnt.toHuman()} ${transactionReceiptToString(this.transactionReceipt, this.config.explorerUrl)}`;
    }
}
exports.TransactionResult = TransactionResult;
function transactionReceiptToString(tx, explorerUrl) {
    return `${tx.signer_id} -> ${tx.receiver_id} Nonce: ${tx.nonce} Hash: ${explorerUrl ? explorerUrl + '/' : ''}${tx.hash} Actions:\n${tx.actions.map(a => JSON.stringify(a)).join('\n')}`;
}
class TransactionError extends Error {
    constructor(result) {
        super(JSON.stringify(result));
    }
    parse() {
        return JSON.parse(this.message);
    }
}
exports.TransactionError = TransactionError;
//# sourceMappingURL=transaction-result.js.map
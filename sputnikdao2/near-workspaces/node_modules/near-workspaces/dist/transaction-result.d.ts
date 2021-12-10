import { Gas } from 'near-units';
import { Action, ClientConfig, ExecutionError, ExecutionOutcome, ExecutionOutcomeWithId, ExecutionStatus, ExecutionStatusBasic, FinalExecutionOutcome, FinalExecutionStatus, FinalExecutionStatusBasic, PublicKey } from './types';
export declare class PromiseOutcome {
    outcome: ExecutionOutcome;
    constructor(outcome: ExecutionOutcome);
    get errors(): Array<Record<string, unknown>>;
    get status(): ExecutionStatus | ExecutionStatusBasic;
    get succeeded(): boolean;
    get isFailure(): boolean;
    get executionStatus(): ExecutionStatus;
    parseResult(): any;
    get SuccessValue(): string | undefined;
    get executionError(): ExecutionError | undefined;
    get errorMessage(): string | undefined;
    get errorType(): string | undefined;
    get logs(): string[];
    get gas_burnt(): Gas;
}
export declare class TransactionResult {
    readonly result: FinalExecutionOutcome;
    readonly startMs: number;
    readonly endMs: number;
    private readonly config;
    constructor(result: FinalExecutionOutcome, startMs: number, endMs: number, config: ClientConfig);
    get durationMs(): number;
    get outcomesWithId(): ExecutionOutcomeWithId[];
    get receipts_outcomes(): PromiseOutcome[];
    get outcome(): ExecutionOutcome[];
    get outcomes(): ExecutionOutcome[];
    get logs(): string[];
    get transactionReceipt(): TransactionReceipt;
    get errors(): ExecutionError[];
    get status(): FinalExecutionStatus | FinalExecutionStatusBasic;
    get succeeded(): boolean;
    get SuccessValue(): string | null;
    get failed(): boolean;
    get Failure(): ExecutionError | null;
    logsContain(pattern: string | RegExp): boolean;
    findLogs(pattern: string | RegExp): string[];
    promiseValuesContain(pattern: string | RegExp): boolean;
    findPromiseValues(pattern: string | RegExp): string[];
    get finalExecutionStatus(): FinalExecutionStatus;
    get promiseErrors(): ExecutionError[];
    get promiseSuccessValues(): string[];
    get promiseErrorMessages(): string[];
    get gas_burnt(): Gas;
    promiseErrorMessagesContain(pattern: string | RegExp): boolean;
    parseResult<T>(): T;
    parsedPromiseResults(): any[];
    summary(): string;
}
export interface TransactionReceipt {
    actions: Action[];
    hash: string;
    nonce: number;
    public_key: PublicKey;
    receiver_id: string;
    signature: string;
    signer_id: string;
}
export declare class TransactionError extends Error {
    constructor(result: TransactionResult);
    parse(): ExecutionOutcome;
}
export declare type TxResult = TransactionResult;
//# sourceMappingURL=transaction-result.d.ts.map
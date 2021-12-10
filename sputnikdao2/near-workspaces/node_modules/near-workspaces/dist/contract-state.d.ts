/// <reference types="node" />
import { Buffer } from 'buffer';
export declare class ContractState {
    private readonly data;
    constructor(dataArray: Array<{
        key: Buffer;
        value: Buffer;
    }>);
    get_raw(key: string): Buffer;
    get(key: string, borshSchema?: {
        type: any;
        schema: any;
    }): any;
}
//# sourceMappingURL=contract-state.d.ts.map
#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const process_1 = __importDefault(require("process"));
const parse_1 = require("./parse");
const HELP = `Parse and format NEAR tokens and gas units. Examples:

    near-units 10 N                             # => 10000000000000000000000000
    near-units -h 10000000000000000000000000 yN # => 10 N
    near-units 50 Tgas                          # => 50000000000000
    near-units -h 50000000000000 gas            # => 50 Tgas
`;
let args = process_1.default.argv.slice(2);
if (args.length === 0) {
    console.log(HELP);
    process_1.default.exit(0);
}
let wantsHuman = false;
if (args.length > 1 && args.includes('-h')) {
    wantsHuman = true;
    args = args.filter((x) => x !== '-h');
}
const input = args.join('');
if (input === '--help') {
    console.log(HELP);
    process_1.default.exit(0);
}
try {
    if (wantsHuman) {
        console.log((0, parse_1.parse)(input).toHuman());
    }
    else {
        console.log((0, parse_1.parse)(input).toJSON());
    }
}
catch (error) {
    if (error instanceof Error) {
        console.error(error.message);
    }
    else {
        console.error(error);
    }
    process_1.default.exit(1);
}
//# sourceMappingURL=cli.js.map
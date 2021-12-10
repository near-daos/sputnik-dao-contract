"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const getBinary_1 = require("./getBinary");
async function run() {
    try {
        const bin = await (0, getBinary_1.getBinary)();
        if (process.argv.length < 3) {
            process.argv.push("--help");
        }
        bin.runAndExit();
    }
    catch (err) {
        console.error(err);
        process.exit(1);
    }
}
run();

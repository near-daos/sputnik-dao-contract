"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getBinary = exports.AWSUrl = void 0;
const _1 = require(".");
const path_1 = require("path");
const os = require("os");
function getPlatform() {
    const type = os.type();
    const arch = os.arch();
    if ((type === "Linux" || type === "Darwin") && arch === "x64") {
        return [type, "x86_64"];
    }
    throw new Error(`Unsupported platform: ${type} ${arch}`);
}
function AWSUrl() {
    const [platform, arch] = getPlatform();
    return `https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/${platform}-${arch}/master/2c9375ee5ee307c2ce870c7dbd25eefd84fe8c36/near-sandbox.tar.gz`;
}
exports.AWSUrl = AWSUrl;
function getBinary(name = "near-sandbox") {
    if (!process.env["NEAR_SANDBOX_BIN_PATH"]) {
        process.env["NEAR_SANDBOX_BINARY_PATH"] = (0, path_1.join)(os.homedir(), ".near", "sandbox");
    }
    // Will use version after publishing to AWS
    // const version = require("./package.json").version;
    const fromEnv = process.env["SANDBOX_ARTIFACT_URL"];
    const urls = [AWSUrl()];
    if (fromEnv) {
        urls.unshift(fromEnv);
    }
    return _1.Binary.create(name, urls);
}
exports.getBinary = getBinary;

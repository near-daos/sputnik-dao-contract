"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rm = exports.inherit = exports.searchPath = exports.fileExists = void 0;
const promises_1 = require("fs/promises");
const path_1 = require("path");
async function fileExists(s) {
    try {
        const f = await (0, promises_1.stat)(s);
        return f.isFile();
    }
    catch {
        return false;
    }
}
exports.fileExists = fileExists;
async function searchPath(filename) {
    const binPath = process.env["NEAR_SANDBOX_BINARY_PATH"];
    if (binPath &&
        binPath.length > 0 &&
        (await fileExists((0, path_1.join)(binPath, filename)))) {
        return binPath;
    }
    return undefined;
}
exports.searchPath = searchPath;
exports.inherit = "inherit";
async function rm(path) {
    try {
        await (0, promises_1.rm)(path);
    }
    catch (e) { }
}
exports.rm = rm;

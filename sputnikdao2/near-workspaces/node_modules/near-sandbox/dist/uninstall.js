"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const getBinary_1 = require("./getBinary");
(0, getBinary_1.getBinary)().then(async (bin) => await bin.uninstall());

var showOutput = false;
var output = function () {
    var input = [];
    for (var _i = 0; _i < arguments.length; _i++) {
        input[_i] = arguments[_i];
    }
    if (showOutput === true) {
        console.log.apply(console.log, input);
    }
};
/**
 * Get a number from within a range
 * @param {number} min
 * @param {number} max
 * @returns {number}
 */
var randomFromRange = function (min, max) {
    if (min === void 0) { min = 1; }
    if (max === void 0) { max = 65535; }
    return +Math.floor(Math.random() * (max - min + 1)) + min;
};
/**
 * Get a range of unique numbers
 * @param {number} howMany
 * @param {number[]} notIn
 * @returns {number[]}
 */
var getUniqueNumbers = function (howMany, notIn) {
    if (howMany === void 0) { howMany = 1; }
    if (notIn === void 0) { notIn = []; }
    var storeNumbers = [];
    var min = 1;
    var max = 65535;
    var randomNr = randomFromRange(min, max);
    if (howMany > max - min) {
        return storeNumbers;
    }
    if (storeNumbers.length < howMany &&
        storeNumbers.indexOf(randomNr) === -1 &&
        notIn.indexOf(randomNr) === -1) {
        storeNumbers.push(randomNr);
    }
    if (storeNumbers.length < howMany) {
        storeNumbers = storeNumbers.concat(getUniqueNumbers(howMany - storeNumbers.length, storeNumbers.concat(notIn)));
    }
    return storeNumbers.sort(function (a, b) { return a - b; });
};
/**
 * Returns the current port if available or the next one available by incrementing the port
 * @param {number} port
 * @param {string} host
 * @returns {Promise<number>}
 */
var nextAvailable = function (port, host) {
    if (port === void 0) { port = 80; }
    if (host === void 0) { host = '0.0.0.0'; }
    return new Promise(function (resolve) {
        isFreePort(port, host)
            .then(function (portStatus) {
            var port = portStatus[0], status = portStatus[2];
            if (status) {
                resolve(port);
            }
            else {
                resolve(nextAvailable(++port, host));
            }
        }).catch(output);
    });
};
/**
 * Get a number of guaranteed free ports available for a host
 * @param {number} howMany
 * @param {string} host
 * @param {number[]} freePorts
 * @returns {Promise<number[]>}
 */
var getFreePorts = function (howMany, host, freePorts) {
    if (howMany === void 0) { howMany = 1; }
    if (host === void 0) { host = "0.0.0.0"; }
    if (freePorts === void 0) { freePorts = []; }
    return new Promise(function (resolve) {
        var uniqueNumbers = getUniqueNumbers(howMany);
        var storeFreePorts = freePorts.slice();
        var stackPromises = [];
        uniqueNumbers.forEach(function (port) {
            stackPromises.push(isFreePort(port, host));
        });
        Promise
            .all(stackPromises)
            .then(function (listStatus) {
            var filteredArrays = listStatus.filter(function (item) { return item[2] !== false; }).map(function (item) { return item[0]; });
            filteredArrays.forEach(function (item) {
                if (storeFreePorts.length < howMany &&
                    storeFreePorts.indexOf(item) === -1 &&
                    freePorts.indexOf(item) === -1) {
                    storeFreePorts.push(item);
                }
            });
            if (storeFreePorts.length < howMany) {
                resolve(storeFreePorts.concat(getUniqueNumbers(howMany - storeFreePorts.length, storeFreePorts.concat(freePorts))));
            }
            else {
                resolve(storeFreePorts);
            }
        }).catch(output);
    });
};
/**
 * Check if a port is free on a certain host
 * @param {number} port
 * @param {string} host
 * @returns {Promise<[number , string , boolean]>}
 */
var isFreePort = function (port, host) {
    if (port === void 0) { port = 80; }
    if (host === void 0) { host = '0.0.0.0'; }
    return new Promise(function (resolve) {
        var net = require('net');
        if (!net.isIPv4(host) || port < 0 || port > 65535) {
            resolve([port, host, false]);
        }
        else {
            var server_1 = net.createServer();
            server_1.on('error', function () { return resolve([port, host, false]); });
            server_1.listen(port, host);
            server_1.on('listening', function () {
                server_1.close();
                server_1.unref();
            });
            server_1.on('close', function () { return resolve([port, host, true]); });
        }
    });
};
exports.isFreePort = isFreePort;
exports.getFreePorts = getFreePorts;
exports.nextAvailable = nextAvailable;

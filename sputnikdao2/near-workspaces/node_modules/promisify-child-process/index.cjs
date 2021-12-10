"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.promisifyChildProcess = promisifyChildProcess;
exports.spawn = spawn;
exports.fork = fork;
exports.execFile = exports.exec = void 0;

const child_process = require('child_process');

const bindFinally = promise => (handler // don't assume we're running in an environment with Promise.finally
) => promise.then(async value => {
  await handler();
  return value;
}, async reason => {
  await handler();
  throw reason;
});

function joinChunks(chunks, encoding) {
  if (chunks[0] instanceof Buffer) {
    const buffer = Buffer.concat(chunks);
    if (encoding) return buffer.toString(encoding);
    return buffer;
  }

  return chunks.join('');
}

function promisifyChildProcess(child, options = {}) {
  const _promise = new Promise((resolve, reject) => {
    const {
      encoding,
      killSignal
    } = options;
    const captureStdio = encoding != null || options.maxBuffer != null;
    const maxBuffer = options.maxBuffer != null ? options.maxBuffer : 200 * 1024;
    let error;
    let bufferSize = 0;
    const stdoutChunks = [];
    const stderrChunks = [];

    const capture = chunks => data => {
      const remaining = Math.max(0, maxBuffer - bufferSize);
      const byteLength = Buffer.byteLength(data, 'utf8');
      bufferSize += Math.min(remaining, byteLength);

      if (byteLength > remaining) {
        error = new Error(`maxBuffer size exceeded`); // $FlowFixMe

        child.kill(killSignal ? killSignal : 'SIGTERM');
        data = data.slice(0, remaining);
      }

      chunks.push(data);
    };

    if (captureStdio) {
      if (child.stdout) child.stdout.on('data', capture(stdoutChunks));
      if (child.stderr) child.stderr.on('data', capture(stderrChunks));
    }

    child.on('error', reject);

    function done(code, signal) {
      if (!error) {
        if (code != null && code !== 0) {
          error = new Error(`Process exited with code ${code}`);
        } else if (signal != null) {
          error = new Error(`Process was killed with ${signal}`);
        }
      }

      function defineOutputs(obj) {
        obj.code = code;
        obj.signal = signal;

        if (captureStdio) {
          obj.stdout = joinChunks(stdoutChunks, encoding);
          obj.stderr = joinChunks(stderrChunks, encoding);
        } else {
          const warn = prop => ({
            configurable: true,
            enumerable: true,

            get() {
              /* eslint-disable no-console */
              console.error(new Error(`To get ${prop} from a spawned or forked process, set the \`encoding\` or \`maxBuffer\` option`).stack.replace(/^Error/, 'Warning'));
              /* eslint-enable no-console */

              return null;
            }

          });

          Object.defineProperties(obj, {
            stdout: warn('stdout'),
            stderr: warn('stderr')
          });
        }
      }

      const finalError = error;

      if (finalError) {
        defineOutputs(finalError);
        reject(finalError);
      } else {
        const output = {};
        defineOutputs(output);
        resolve(output);
      }
    }

    child.on('close', done);
  });

  return Object.create(child, {
    then: {
      value: _promise.then.bind(_promise)
    },
    catch: {
      value: _promise.catch.bind(_promise)
    },
    finally: {
      value: bindFinally(_promise)
    }
  });
}

function spawn(command, args, options) {
  return promisifyChildProcess(child_process.spawn(command, args, options), Array.isArray(args) ? options : args);
}

function fork(module, args, options) {
  return promisifyChildProcess(child_process.fork(module, args, options), Array.isArray(args) ? options : args);
}

function promisifyExecMethod(method) {
  return (...args) => {
    let child;

    const _promise = new Promise((resolve, reject) => {
      child = method(...args, (err, stdout, stderr) => {
        if (err) {
          err.stdout = stdout;
          err.stderr = stderr;
          reject(err);
        } else {
          resolve({
            code: 0,
            signal: null,
            stdout,
            stderr
          });
        }
      });
    });

    if (!child) {
      throw new Error('unexpected error: child has not been initialized');
    }

    return Object.create(child, {
      then: {
        value: _promise.then.bind(_promise)
      },
      catch: {
        value: _promise.catch.bind(_promise)
      },
      finally: {
        value: bindFinally(_promise)
      }
    });
  };
}

const exec = promisifyExecMethod(child_process.exec);
exports.exec = exec;
const execFile = promisifyExecMethod(child_process.execFile);
exports.execFile = execFile;

// @flow

import type { ChildProcess } from 'child_process'
const child_process = require('child_process')

export type ChildProcessOutput = {
  stdout?: ?(string | Buffer),
  stderr?: ?(string | Buffer),
  code?: number | null,
  signal?: string | null,
}

export interface ExitReason {
  code?: number | null;
  signal?: string | null;
}

export type ErrorWithOutput = Error & ChildProcessOutput

export type ChildProcessPromise = ChildProcess & Promise<ChildProcessOutput>

type PromisifyChildProcessBaseOpts = {
  encoding?: $PropertyType<child_process$spawnSyncOpts, 'encoding'>,
  killSignal?: $PropertyType<child_process$spawnSyncOpts, 'killSignal'>,
  maxBuffer?: $PropertyType<child_process$spawnSyncOpts, 'maxBuffer'>,
}

export type SpawnOpts = child_process$spawnOpts & PromisifyChildProcessBaseOpts
export type ForkOpts = child_process$forkOpts & PromisifyChildProcessBaseOpts

const bindFinally = <T>(promise: Promise<T>) => (
  handler: () => mixed
): Promise<T> =>
  // don't assume we're running in an environment with Promise.finally
  promise.then(
    async (value: any): any => {
      await handler()
      return value
    },
    async (reason: any): any => {
      await handler()
      throw reason
    }
  )

function joinChunks(
  chunks: $ReadOnlyArray<string | Buffer>,
  encoding: ?string
): string | Buffer {
  if (chunks[0] instanceof Buffer) {
    const buffer = Buffer.concat((chunks: any))
    if (encoding) return buffer.toString((encoding: any))
    return buffer
  }
  return chunks.join('')
}

export function promisifyChildProcess(
  child: ChildProcess,
  options: PromisifyChildProcessBaseOpts = {}
): ChildProcessPromise {
  const _promise = new Promise(
    (
      resolve: (result: ChildProcessOutput) => void,
      reject: (error: ErrorWithOutput) => void
    ) => {
      const { encoding, killSignal } = options
      const captureStdio = encoding != null || options.maxBuffer != null
      const maxBuffer =
        options.maxBuffer != null ? options.maxBuffer : 200 * 1024

      let error: ?ErrorWithOutput
      let bufferSize = 0
      const stdoutChunks: Array<string | Buffer> = []
      const stderrChunks: Array<string | Buffer> = []
      const capture = (chunks: Array<string | Buffer>) => (
        data: string | Buffer
      ) => {
        const remaining = Math.max(0, maxBuffer - bufferSize)
        const byteLength = Buffer.byteLength(data, 'utf8')
        bufferSize += Math.min(remaining, byteLength)
        if (byteLength > remaining) {
          error = new Error(`maxBuffer size exceeded`)
          // $FlowFixMe
          child.kill(killSignal ? killSignal : 'SIGTERM')
          data = data.slice(0, remaining)
        }
        chunks.push(data)
      }
      if (captureStdio) {
        if (child.stdout) child.stdout.on('data', capture(stdoutChunks))
        if (child.stderr) child.stderr.on('data', capture(stderrChunks))
      }

      child.on('error', reject)
      function done(code: ?number, signal: ?string) {
        if (!error) {
          if (code != null && code !== 0) {
            error = new Error(`Process exited with code ${code}`)
          } else if (signal != null) {
            error = new Error(`Process was killed with ${signal}`)
          }
        }
        function defineOutputs(obj: Object) {
          obj.code = code
          obj.signal = signal
          if (captureStdio) {
            obj.stdout = joinChunks(stdoutChunks, encoding)
            obj.stderr = joinChunks(stderrChunks, encoding)
          } else {
            const warn = (prop: 'stdout' | 'stderr') => ({
              configurable: true,
              enumerable: true,
              get(): any {
                /* eslint-disable no-console */
                console.error(
                  new Error(
                    `To get ${prop} from a spawned or forked process, set the \`encoding\` or \`maxBuffer\` option`
                  ).stack.replace(/^Error/, 'Warning')
                )
                /* eslint-enable no-console */
                return null
              },
            })
            Object.defineProperties(obj, {
              stdout: warn('stdout'),
              stderr: warn('stderr'),
            })
          }
        }
        const finalError: ?ErrorWithOutput = error
        if (finalError) {
          defineOutputs(finalError)
          reject(finalError)
        } else {
          const output: ChildProcessOutput = ({}: any)
          defineOutputs(output)
          resolve(output)
        }
      }
      child.on('close', done)
    }
  )
  return (Object.create(child, {
    then: { value: _promise.then.bind(_promise) },
    catch: { value: _promise.catch.bind(_promise) },
    finally: {
      value: bindFinally(_promise),
    },
  }): any)
}

export function spawn(
  command: string,
  args?: Array<string> | child_process$spawnOpts,
  options?: SpawnOpts
): ChildProcessPromise {
  return promisifyChildProcess(
    child_process.spawn(command, args, options),
    ((Array.isArray(args) ? options : args): any)
  )
}

export function fork(
  module: string,
  args?: Array<string> | child_process$forkOpts,
  options?: ForkOpts
): ChildProcessPromise {
  return promisifyChildProcess(
    child_process.fork(module, args, options),
    ((Array.isArray(args) ? options : args): any)
  )
}

function promisifyExecMethod(method: any): any {
  return (...args: Array<any>): ChildProcessPromise => {
    let child: ?ChildProcess
    const _promise = new Promise(
      (
        resolve: (output: ChildProcessOutput) => void,
        reject: (error: ErrorWithOutput) => void
      ) => {
        child = method(
          ...args,
          (
            err: ?ErrorWithOutput,
            stdout: ?(Buffer | string),
            stderr: ?(Buffer | string)
          ) => {
            if (err) {
              err.stdout = stdout
              err.stderr = stderr
              reject(err)
            } else {
              resolve({ code: 0, signal: null, stdout, stderr })
            }
          }
        )
      }
    )
    if (!child) {
      throw new Error('unexpected error: child has not been initialized')
    }
    return (Object.create((child: any), {
      then: { value: _promise.then.bind(_promise) },
      catch: { value: _promise.catch.bind(_promise) },
      finally: { value: bindFinally(_promise) },
    }): any)
  }
}

export const exec: (
  command: string,
  options?: child_process$execOpts
) => ChildProcessPromise = promisifyExecMethod(child_process.exec)

export const execFile: (
  file: string,
  args?: Array<string> | child_process$execFileOpts,
  options?: child_process$execOpts
) => ChildProcessPromise = promisifyExecMethod(child_process.execFile)

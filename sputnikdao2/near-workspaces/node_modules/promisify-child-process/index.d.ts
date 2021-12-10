import {
  ChildProcess,
  SpawnOptions,
  ExecOptions,
  ExecFileOptions,
  ForkOptions,
} from 'child_process'

export interface Output {
  stdout?: string | Buffer | null | undefined
  stderr?: string | Buffer | null | undefined
  code?: number | null
  signal?: string | null
}

export type ErrorWithOutput = Error & Output

export type ChildProcessPromise = ChildProcess & Promise<Output>

export interface PromisifyChildProcessOptions {
  encoding?: string
  killSignal?: string
  maxBuffer?: number
}

export type PromisifySpawnOptions = SpawnOptions & PromisifyChildProcessOptions

export type PromisifyForkOptions = ForkOptions & PromisifyChildProcessOptions

export function promisifyChildProcess(
  child: ChildProcess,
  options?: PromisifyChildProcessOptions
): ChildProcessPromise

export function spawn(
  command: string,
  args: Array<string>,
  options?: PromisifySpawnOptions
): ChildProcessPromise
export function spawn(
  command: string,
  options?: PromisifySpawnOptions
): ChildProcessPromise

export function fork(
  module: string,
  args: Array<string>,
  options?: PromisifyForkOptions
): ChildProcessPromise
export function fork(
  module: string,
  options?: PromisifyForkOptions
): ChildProcessPromise

export function exec(
  command: string,
  options?: ExecOptions
): ChildProcessPromise

export function execFile(
  file: string,
  args: Array<string>,
  options?: ExecFileOptions
): ChildProcessPromise
export function execFile(
  file: string,
  options?: ExecFileOptions
): ChildProcessPromise

import { ChildProcess, spawn } from "child_process";
import * as fs from "fs";
import * as net from "net";
import * as path from "path";

export interface RecorderConfig {
  bin: string;
  socket: string;
}

export interface StartRecorderOptions extends RecorderConfig {
  args: string[];
  connectTimeoutMs: number;
}

export interface RecorderHandle {
  stop(): Promise<void>;
}

function canConnect(socket: string): Promise<boolean> {
  return new Promise((resolve) => {
    const s = net.createConnection(socket);
    const done = (ok: boolean) => {
      s.removeAllListeners();
      s.destroy();
      resolve(ok);
    };
    s.once("connect", () => done(true));
    s.once("error", () => done(false));
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

/** Reads and validates the recorder location from the environment. No defaults. */
export function readRecorderConfig(): RecorderConfig {
  const bin = process.env.SE_BIN;
  const socket = process.env.SE_CONTROL;
  if (!bin) throw new Error("SE_BIN must be set to the path of the stealing-eye binary");
  if (!socket) throw new Error("SE_CONTROL must be set to the path of the control socket");
  if (!path.isAbsolute(bin)) throw new Error(`SE_BIN must be an absolute path: ${bin}`);
  if (!path.isAbsolute(socket)) throw new Error(`SE_CONTROL must be an absolute path: ${socket}`);
  let stat: fs.Stats;
  try {
    stat = fs.statSync(bin);
  } catch {
    throw new Error(`SE_BIN does not exist: ${bin}`);
  }
  if (!stat.isFile()) throw new Error(`SE_BIN is not a file: ${bin}`);
  try {
    fs.accessSync(bin, fs.constants.X_OK);
  } catch {
    throw new Error(`SE_BIN is not executable: ${bin}`);
  }
  return { bin, socket };
}

/** Spawns the recorder in control mode and waits for its control socket to bind. */
export async function startRecorder(opts: StartRecorderOptions): Promise<RecorderHandle> {
  if (await canConnect(opts.socket)) {
    throw new Error(`recorder already running at ${opts.socket}`);
  }

  const child = spawn(opts.bin, ["--control", opts.socket, ...opts.args], { stdio: "inherit" });
  const spawnError = await new Promise<Error | null>((resolve) => {
    child.once("error", (e) => resolve(e));
    child.once("spawn", () => resolve(null));
  });
  if (spawnError) throw new Error(`failed to spawn recorder ${opts.bin}: ${spawnError.message}`);

  const deadline = Date.now() + opts.connectTimeoutMs;
  for (;;) {
    if (child.exitCode !== null || child.signalCode !== null) {
      throw new Error(`recorder exited before binding ${opts.socket} (code ${child.exitCode})`);
    }
    if (await canConnect(opts.socket)) break;
    if (Date.now() >= deadline) {
      child.kill("SIGKILL");
      throw new Error(`recorder did not bind ${opts.socket} within ${opts.connectTimeoutMs}ms`);
    }
    await sleep(100);
  }

  return { stop: () => stopRecorder(child, opts.socket) };
}

async function stopRecorder(child: ChildProcess, socket: string): Promise<void> {
  await new Promise<void>((resolve) => {
    const s = net.createConnection(socket);
    s.once("connect", () => s.end(`${JSON.stringify({ cmd: "shutdown" })}\n`, () => resolve()));
    s.once("error", () => resolve());
  });

  await new Promise<void>((resolve) => {
    if (child.exitCode !== null || child.signalCode !== null) return resolve();
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      resolve();
    }, 5000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
  });

  try {
    fs.unlinkSync(socket);
  } catch {
    // socket already gone
  }
}

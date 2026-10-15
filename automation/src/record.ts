#!/usr/bin/env node
import { execFileSync } from "child_process";
import { Control, Credentials, envCredentials, envLoginOptions, launchBrowser, validateEnv } from "./shared";
import { runAutomation } from "./run";
import { readRecorderConfig, startRecorder } from "./recorder";

interface Opts {
  site?: string;
  output: string;
  fps: number;
  minClip: number;
  minDuration: number;
  minAreaRatio: number;
  stopDebounceMs: number;
  audio: boolean;
}

interface Signal {
  type: "play" | "pause" | "ended";
  id: number;
  duration?: number;
  title?: string;
  src?: string;
}

const REQUIRED_FLAGS = [
  "--output",
  "--fps",
  "--min-clip",
  "--min-duration",
  "--min-area",
  "--stop-debounce",
] as const;

function parseArgs(argv: string[]): Opts {
  const seen = new Set<string>();
  let site: string | undefined;
  let output: string | undefined;
  let fps: number | undefined;
  let minClip: number | undefined;
  let minDuration: number | undefined;
  let minAreaRatio: number | undefined;
  let stopDebounceMs: number | undefined;
  let audio: boolean | undefined;

  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    const num = (): number => {
      const v = Number(argv[++i]);
      if (!Number.isFinite(v)) throw new Error(`flag ${a} requires a number`);
      return v;
    };
    if (a === "--site") site = argv[++i];
    else if (a === "--output") output = argv[++i];
    else if (a === "--fps") fps = num();
    else if (a === "--min-clip") minClip = num();
    else if (a === "--min-duration") minDuration = num();
    else if (a === "--min-area") minAreaRatio = num();
    else if (a === "--stop-debounce") stopDebounceMs = num();
    else if (a === "--audio") audio = true;
    else if (a === "--no-audio") audio = false;
    else throw new Error(`unknown flag ${a}`);
    if (a.startsWith("--") && a !== "--site") seen.add(a);
  }

  const missing = REQUIRED_FLAGS.filter((f) => !seen.has(f));
  if (missing.length) throw new Error(`missing required flags: ${missing.join(", ")}`);
  if (audio === undefined) throw new Error("missing required flag: --audio or --no-audio");

  return {
    site,
    output: output!,
    fps: fps!,
    minClip: minClip!,
    minDuration: minDuration!,
    minAreaRatio: minAreaRatio!,
    stopDebounceMs: stopDebounceMs!,
    audio,
  };
}

const INIT_SCRIPT = (minDuration: number, minArea: number): string => `
(() => {
  const send = (msg) => { try { window.__seSignal && window.__seSignal(msg); } catch (e) {} };
  const ids = new WeakMap();
  let n = 0;
  const idOf = (v) => { if (!ids.has(v)) ids.set(v, ++n); return ids.get(v); };
  const mainPlayer = (v) => {
    if (!v || v.muted || v.ended) return false;
    const r = v.getBoundingClientRect();
    const area = r.width * r.height;
    const vp = window.innerWidth * window.innerHeight || 1;
    const longEnough = !isFinite(v.duration) || v.duration > ${minDuration};
    return longEnough && area > ${minArea} * vp;
  };
  document.addEventListener("play", (e) => {
    const v = e.target;
    if (mainPlayer(v)) send({ type: "play", id: idOf(v), duration: v.duration, title: document.title, src: location.href });
  }, true);
  document.addEventListener("pause", (e) => {
    const v = e.target;
    if (mainPlayer(v)) send({ type: "pause", id: idOf(v), title: document.title });
  }, true);
  document.addEventListener("ended", (e) => {
    const v = e.target;
    if (mainPlayer(v)) send({ type: "ended", id: idOf(v), title: document.title });
  }, true);
})();
`;

function findFirefoxWindow(titleHint?: string): string {
  let out: string;
  try {
    out = execFileSync("wmctrl", ["-l"], { encoding: "utf8" });
  } catch (e) {
    throw new Error(`wmctrl failed: ${(e as Error).message}`);
  }
  const rows = out
    .split("\n")
    .filter(Boolean)
    .map((l) => {
      const m = l.match(/^(0x[0-9a-fA-F]+)\s+\S+\s+\S+\s+(.*)$/);
      return m ? { id: m[1], title: m[2] } : null;
    })
    .filter((r): r is { id: string; title: string } => r !== null);
  const hint = (titleHint || "").toLowerCase();
  const match = hint
    ? rows.find((r) => r.title.toLowerCase().includes(hint))
    : rows.find((r) => /firefox/i.test(r.title));
  if (!match) {
    throw new Error(`no Firefox window matched '${titleHint ?? ""}'`);
  }
  return match.id;
}

async function main(): Promise<void> {
  const opts = parseArgs(process.argv);
  validateEnv();
  const creds: Credentials = envCredentials();

  const rec = readRecorderConfig();
  const recorder = await startRecorder({
    bin: rec.bin,
    socket: rec.socket,
    args: ["-o", opts.output, "--fps", String(opts.fps), "--min-clip", String(opts.minClip), ...(opts.audio ? [] : ["--no-audio"])],
    connectTimeoutMs: 10000,
  });

  const control = new Control(rec.socket);
  await control.connect();
  console.log(`connected to ${rec.socket}`);

  const browser = await launchBrowser();
  const context = await browser.newContext();
  await context.addInitScript(INIT_SCRIPT(opts.minDuration, opts.minAreaRatio));

  const playing = new Set<number>();
  let recording = false;
  let stopTimer: NodeJS.Timeout | null = null;
  let shuttingDown = false;

  const shutdown = (code: number): void => {
    if (shuttingDown) return;
    shuttingDown = true;
    try {
      if (recording) control.send({ cmd: "stop" });
    } catch {
      // control socket may be gone
    }
    recorder
      .stop()
      .catch((e: Error) => console.error(`stopping recorder: ${e.message}`))
      .finally(() => browser.close().finally(() => process.exit(code)));
  };

  function handleSignal(payload: Signal): void {
    if (!payload || !payload.type) return;
    const id = payload.id;
    if (payload.type === "play") {
      playing.add(id);
      if (stopTimer) {
        clearTimeout(stopTimer);
        stopTimer = null;
      }
      if (!recording) {
        const win = findFirefoxWindow(payload.title);
        recording = true;
        control.send({ cmd: "start", window: win, label: payload.title || undefined });
        console.log(`start clip (window=${win})`);
      }
    } else if (payload.type === "pause" || payload.type === "ended") {
      playing.delete(id);
      if (playing.size === 0 && recording && !stopTimer) {
        stopTimer = setTimeout(() => {
          stopTimer = null;
          if (playing.size === 0 && recording) {
            recording = false;
            control.send({ cmd: "stop" });
            console.log("stop clip");
          }
        }, opts.stopDebounceMs);
      }
    }
  }

  await context.exposeBinding("__seSignal", (_source, payload: Signal) => {
    try {
      handleSignal(payload);
    } catch (e) {
      console.error((e as Error).message);
      shutdown(1);
    }
  });

  const page = await context.newPage();
  const loginOpts = { ...envLoginOptions(), ...(opts.site ? { site: opts.site } : {}) };
  const emit = (name: string, data?: Record<string, unknown>): void => {
    control.send({ cmd: "event", name, data });
  };
  try {
    await runAutomation({ page, creds, opts: loginOpts, emit });
  } catch (e) {
    console.error((e as Error).message);
    shutdown(1);
    return;
  }

  page.on("close", () => {
    if (recording) {
      recording = false;
      control.send({ cmd: "stop" });
    }
  });
  browser.on("disconnected", () => shutdown(0));

  process.on("SIGINT", () => shutdown(0));
  process.on("SIGTERM", () => shutdown(0));

  console.log("watching for <video> playback (Ctrl-C to quit)");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

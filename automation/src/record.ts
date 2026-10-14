#!/usr/bin/env node
import { execFileSync } from "child_process";
import { Page } from "playwright";
import { Control, Credentials, envCredentials, envLoginOptions, launchBrowser } from "./shared";
import { runLoginMachine } from "./machines/login";

interface Opts {
  socket: string;
  url: string | null;
  minDuration: number;
  minAreaRatio: number;
  stopDebounceMs: number;
}

interface Signal {
  type: "play" | "pause" | "ended";
  id: number;
  duration?: number;
  title?: string;
  src?: string;
}

function parseArgs(argv: string[]): Opts {
  const opts: Opts = {
    socket: process.env.SE_CONTROL || "/tmp/stealing-eye.sock",
    url: null,
    minDuration: 20,
    minAreaRatio: 0.4,
    stopDebounceMs: 1000,
  };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--socket") opts.socket = argv[++i];
    else if (a === "--min-duration") opts.minDuration = Number(argv[++i]);
    else if (a === "--min-area") opts.minAreaRatio = Number(argv[++i]);
    else if (a === "--stop-debounce") opts.stopDebounceMs = Number(argv[++i]);
    else if (a.startsWith("-")) throw new Error(`unknown flag ${a}`);
    else opts.url = a;
  }
  return opts;
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

function findFirefoxWindow(titleHint?: string): string | null {
  let out: string;
  try {
    out = execFileSync("wmctrl", ["-l"], { encoding: "utf8" });
  } catch {
    return null;
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
  const byHint = rows.find((r) => hint && r.title.toLowerCase().includes(hint));
  if (byHint) return byHint.id;
  const firefox = rows.find((r) => /firefox/i.test(r.title));
  return firefox ? firefox.id : null;
}

async function main(): Promise<void> {
  const opts = parseArgs(process.argv);
  const creds: Credentials = envCredentials();
  const hasCreds = Boolean(creds.username && creds.password);
  const control = new Control(opts.socket);

  let connected = true;
  try {
    await control.connect();
    console.log(`connected to ${opts.socket}`);
  } catch (e) {
    if (!hasCreds) {
      console.error(`cannot connect to stealing-eye socket ${opts.socket}: ${(e as Error).message}`);
      process.exit(1);
    }
    connected = false;
    console.warn(
      `cannot connect to stealing-eye socket ${opts.socket} (${(e as Error).message}); continuing in login-only mode`
    );
  }

  const browser = await launchBrowser();
  const context = await browser.newContext();
  await context.exposeBinding("__seSignal", (_source, payload: Signal) => {
    handleSignal(payload);
  });
  await context.addInitScript(INIT_SCRIPT(opts.minDuration, opts.minAreaRatio));

  const playing = new Set<number>();
  let recording = false;
  let stopTimer: NodeJS.Timeout | null = null;

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
        control.send({
          cmd: "start",
          window: win || undefined,
          label: payload.title || undefined,
        });
        console.log(`start clip (window=${win || "whole screen"})`);
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

  const url = opts.url || creds.url;
  const page = await context.newPage();
  if (url) {
    if (hasCreds) {
      const finalState = await runLoginMachine({ page, creds, url, opts: envLoginOptions() });
      if (finalState !== "AUTHENTICATED") {
        console.warn(`login ended in ${finalState}; watching anyway (Ctrl-C to quit)`);
      }
    } else {
      await page.goto(url, { waitUntil: "domcontentloaded" });
    }
  }

  page.on("close", () => {
    if (recording) {
      recording = false;
      control.send({ cmd: "stop" });
    }
  });

  const shutdown = () => {
    try {
      if (recording) control.send({ cmd: "stop" });
      if (connected) control.send({ cmd: "shutdown" });
    } catch {}
    browser.close().finally(() => process.exit(0));
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  console.log("watching for <video> playback (Ctrl-C to quit)");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

#!/usr/bin/env node
"use strict";

// Keep the Playwright browser download inside this repo's node_modules.
process.env.PLAYWRIGHT_BROWSERS_PATH = process.env.PLAYWRIGHT_BROWSERS_PATH || "0";

const net = require("net");
const { execFileSync } = require("child_process");
const { firefox } = require("playwright");

function parseArgs(argv) {
  const opts = {
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

class Control {
  constructor(path) {
    this.path = path;
    this.socket = null;
    this.buffer = "";
  }
  connect() {
    return new Promise((resolve, reject) => {
      const s = net.createConnection(this.path);
      s.on("connect", () => {
        this.socket = s;
        resolve();
      });
      s.on("error", reject);
      s.on("data", (d) => this.onData(d));
    });
  }
  onData(d) {
    this.buffer += d.toString();
    let idx;
    while ((idx = this.buffer.indexOf("\n")) >= 0) {
      const line = this.buffer.slice(0, idx);
      this.buffer = this.buffer.slice(idx + 1);
      if (line.trim()) console.log("<-", line);
    }
  }
  send(cmd) {
    const line = JSON.stringify(cmd) + "\n";
    console.log("->", line.trim());
    if (this.socket) this.socket.write(line);
  }
}

const INIT_SCRIPT = (minDuration, minArea) => `
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

function findFirefoxWindow(titleHint) {
  let out;
  try {
    out = execFileSync("wmctrl", ["-l"], { encoding: "utf8" });
  } catch {
    return null;
  }
  const lines = out.split("\n").filter(Boolean);
  const rows = lines.map((l) => {
    const m = l.match(/^(0x[0-9a-fA-F]+)\s+\S+\s+\S+\s+(.*)$/);
    return m ? { id: m[1], title: m[2] } : null;
  }).filter(Boolean);
  const hint = (titleHint || "").toLowerCase();
  const byHint = rows.find((r) => hint && r.title.toLowerCase().includes(hint));
  if (byHint) return byHint.id;
  const firefox = rows.find((r) => /firefox/i.test(r.title));
  return firefox ? firefox.id : null;
}

async function main() {
  const opts = parseArgs(process.argv);
  const control = new Control(opts.socket);

  try {
    await control.connect();
  } catch (e) {
    console.error(`cannot connect to stealing-eye socket ${opts.socket}: ${e.message}`);
    process.exit(1);
  }
  console.log(`connected to ${opts.socket}`);

  const browser = await firefox.launch({
    headless: false,
    firefoxUserPrefs: {
      "media.autoplay.default": 0,
      "media.autoplay.blocking_policy": 0,
      "media.autoplay.allow-muted": true,
    },
  });

  const context = await browser.newContext();
  await context.exposeBinding("__seSignal", (source, payload) => {
    handleSignal(payload);
  });
  await context.addInitScript(INIT_SCRIPT(opts.minDuration, opts.minAreaRatio));

  const playing = new Set();
  let recording = false;
  let stopTimer = null;

  function handleSignal(payload) {
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

  const page = await context.newPage();
  if (opts.url) await page.goto(opts.url, { waitUntil: "domcontentloaded" });

  page.on("close", () => {
    if (recording) {
      recording = false;
      control.send({ cmd: "stop" });
    }
  });

  const shutdown = () => {
    try {
      if (recording) control.send({ cmd: "stop" });
      control.send({ cmd: "shutdown" });
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

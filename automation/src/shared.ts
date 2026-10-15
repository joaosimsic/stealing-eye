import * as net from "net";
import * as path from "path";
import { firefox, Browser } from "playwright";

// Keep the Playwright browser download inside this repo's node_modules.
process.env.PLAYWRIGHT_BROWSERS_PATH = process.env.PLAYWRIGHT_BROWSERS_PATH || "0";

require("dotenv").config({ path: path.join(__dirname, "..", ".env") });

export interface Credentials {
  email?: string;
  password?: string;
}

export function envCredentials(): Credentials {
  return {
    email: process.env.EMAIL,
    password: process.env.PASSWORD,
  };
}

export class Control {
  private socket: net.Socket | null = null;
  private buffer = "";

  constructor(private readonly path: string) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const s = net.createConnection(this.path);
      s.on("connect", () => {
        this.socket = s;
        resolve();
      });
      s.on("error", reject);
      s.on("data", (d: Buffer) => this.onData(d));
    });
  }

  private onData(d: Buffer): void {
    this.buffer += d.toString();
    let idx: number;
    while ((idx = this.buffer.indexOf("\n")) >= 0) {
      const line = this.buffer.slice(0, idx);
      this.buffer = this.buffer.slice(idx + 1);
      if (line.trim()) console.log("<-", line);
    }
  }

  send(cmd: Record<string, unknown>): void {
    const line = JSON.stringify(cmd) + "\n";
    console.log("->", line.trim());
    if (this.socket) this.socket.write(line);
  }
}

export async function launchBrowser(): Promise<Browser> {
  return firefox.launch({
    headless: false,
    firefoxUserPrefs: {
      "media.autoplay.default": 0,
      "media.autoplay.blocking_policy": 0,
      "media.autoplay.allow-muted": true,
    },
  });
}

export interface LoginEnvOptions {
  site?: string;
}

export function envLoginOptions(): LoginEnvOptions {
  return {
    site: process.env.SITE || undefined,
  };
}

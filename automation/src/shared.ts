import * as net from "net";
import * as path from "path";
import { firefox, Browser, Locator, Page } from "playwright";

// Keep the Playwright browser download inside this repo's node_modules.
process.env.PLAYWRIGHT_BROWSERS_PATH = process.env.PLAYWRIGHT_BROWSERS_PATH || "0";

require("dotenv").config({ path: path.join(__dirname, "..", ".env") });

export interface Credentials {
  url?: string;
  username?: string;
  password?: string;
}

export function envCredentials(): Credentials {
  return {
    url: process.env.URL,
    username: process.env.USERNAME,
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

async function firstVisible(page: Page, selectors: string[]): Promise<Locator | null> {
  for (const selector of selectors) {
    const locator = page.locator(selector).first();
    try {
      if ((await locator.count()) > 0 && (await locator.isVisible())) return locator;
    } catch {
      // selector may be invalid on some pages; keep trying
    }
  }
  return null;
}

const USERNAME_SELECTORS = [
  "input[type=email]",
  "input[name*=user i]",
  "input[name*=email i]",
  "input[id*=user i]",
  "input[id*=email i]",
  "input[type=text]",
];

const PASSWORD_SELECTORS = ["input[type=password]"];

const SUBMIT_SELECTORS = [
  "button[type=submit]",
  "input[type=submit]",
  "form button",
  "button",
];

export async function login(page: Page, creds: Credentials): Promise<void> {
  if (!creds.username || !creds.password) {
    throw new Error("USERNAME and PASSWORD must be set in .env to log in");
  }

  const username = await firstVisible(page, USERNAME_SELECTORS);
  const password = await firstVisible(page, PASSWORD_SELECTORS);
  if (!username || !password) {
    throw new Error("could not find username/password fields on the page");
  }

  await username.fill(creds.username);
  await password.fill(creds.password);

  const submit = await firstVisible(page, SUBMIT_SELECTORS);
  if (submit) {
    await submit.click();
  } else {
    await password.press("Enter");
  }

  try {
    await page.waitForLoadState("networkidle", { timeout: 15000 });
  } catch {
    // some apps never go idle; proceed anyway
  }
  console.log("login submitted");
}

import * as path from "path";
import { Locator, Page } from "playwright";
import { Credentials } from "../shared";
import { runMachine, StateDef, StateName } from "../machine";

export interface LoginOptions {
  destinationUrl?: string;
  successSelector?: string;
  errorSelector?: string;
  timeoutMs?: number;
  pollMs?: number;
}

export interface LoginContext {
  page: Page;
  creds: Credentials;
  url: string;
  opts: LoginOptions;
  loginUrl?: string;
  username?: Locator | null;
  password?: Locator | null;
  submitted?: boolean;
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

async function visible(page: Page, selector?: string): Promise<boolean> {
  if (!selector) return false;
  try {
    const locator = page.locator(selector).first();
    return (await locator.count()) > 0 && (await locator.isVisible());
  } catch {
    return false;
  }
}

async function successSignal(ctx: LoginContext): Promise<boolean> {
  if (await visible(ctx.page, ctx.opts.successSelector)) return true;
  const onLogin = await visible(ctx.page, PASSWORD_SELECTORS[0]);
  return !onLogin && ctx.page.url() !== ctx.loginUrl;
}

async function failureSignal(ctx: LoginContext): Promise<boolean> {
  if (await visible(ctx.page, ctx.opts.errorSelector)) return true;
  return visible(ctx.page, '[role=alert]');
}

const states: Record<StateName, StateDef<LoginContext>> = {
  LOGIN_PAGE: {
    action: async (ctx) => {
      if (ctx.page.url() !== ctx.url) {
        await ctx.page.goto(ctx.url, { waitUntil: "domcontentloaded" });
      }
      ctx.loginUrl = ctx.page.url();
      ctx.username = await firstVisible(ctx.page, USERNAME_SELECTORS);
      ctx.password = await firstVisible(ctx.page, PASSWORD_SELECTORS);
    },
    exit: async (ctx) => Boolean(ctx.username && ctx.password),
    next: "ENTER_CREDENTIALS",
  },

  ENTER_CREDENTIALS: {
    action: async (ctx) => {
      if (!ctx.username || !ctx.password) throw new Error("username/password fields missing");
      await ctx.username.fill(ctx.creds.username ?? "");
      await ctx.password.fill(ctx.creds.password ?? "");
    },
    exit: async (ctx) => {
      if (!ctx.username || !ctx.password) return false;
      return (
        (await ctx.username.inputValue()) === (ctx.creds.username ?? "") &&
        (await ctx.password.inputValue()) === (ctx.creds.password ?? "")
      );
    },
    next: "SUBMIT",
  },

  SUBMIT: {
    action: async (ctx) => {
      const submit = await firstVisible(ctx.page, SUBMIT_SELECTORS);
      const navigation = ctx.page
        .waitForLoadState("networkidle", { timeout: 10000 })
        .catch(() => undefined);
      if (submit) await submit.click();
      else if (ctx.password) await ctx.password.press("Enter");
      else throw new Error("no submit control and no password field");
      await navigation;
      ctx.submitted = true;
    },
    exit: async (ctx) => Boolean(ctx.submitted),
    next: "CHECK_RESULT",
  },

  CHECK_RESULT: {
    exit: async (ctx) => (await successSignal(ctx)) || (await failureSignal(ctx)),
    next: async (ctx) => ((await successSignal(ctx)) ? "AUTHENTICATED" : "LOGIN_FAILED"),
    timeoutMs: 20000,
  },

  AUTHENTICATED: {
    action: async (ctx) => {
      if (ctx.opts.destinationUrl) {
        await ctx.page.goto(ctx.opts.destinationUrl, { waitUntil: "domcontentloaded" });
      }
      console.log(`[login] authenticated at ${ctx.page.url()}`);
    },
    exit: async (ctx) => {
      if (ctx.opts.destinationUrl && ctx.page.url() === ctx.opts.destinationUrl) return true;
      return successSignal(ctx);
    },
    next: "AUTHENTICATED",
    terminal: true,
  },

  LOGIN_FAILED: {
    action: async (ctx) => {
      const shot = path.join(__dirname, "..", "..", "login-failed.png");
      try {
        await ctx.page.screenshot({ path: shot });
      } catch {
        // page may be gone; failure is reported regardless
      }
      console.error(`[login] failed at ${ctx.page.url()} (screenshot: ${shot})`);
    },
    exit: async () => true,
    next: "LOGIN_FAILED",
    terminal: true,
  },
};

export async function runLoginMachine(ctx: LoginContext): Promise<StateName> {
  console.log("[fsm] start LOGIN_PAGE");
  return runMachine(ctx, "LOGIN_PAGE", {
    states,
    failure: "LOGIN_FAILED",
    defaultTimeoutMs: ctx.opts.timeoutMs ?? 15000,
    defaultPollMs: ctx.opts.pollMs ?? 250,
  });
}

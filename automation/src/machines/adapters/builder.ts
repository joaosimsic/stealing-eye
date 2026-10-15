import * as path from "path";
import { StateDef, StateName } from "../../machine";
import { BuiltMachine, LoginContext, SiteAdapter, Step } from "./types";
import { conditionMet, firstVisible, selectorsOf, waitForReady } from "./selectors";

const STEP_TIMEOUT_MS = 15000;
const CHECK_TIMEOUT_MS = 20000;
const SUBMIT_SETTLE_MS = 10000;
const DEFAULT_POLL_MS = 250;

function toArray(value: string | string[]): string[] {
  return typeof value === "string" ? [value] : value;
}

async function runStep(ctx: LoginContext, step: Step): Promise<void> {
  const timeoutMs = step.timeoutMs ?? STEP_TIMEOUT_MS;
  const pollMs = ctx.opts.pollMs ?? DEFAULT_POLL_MS;

  const ready = await waitForReady(ctx, step.ready, timeoutMs, pollMs);
  if (!ready) {
    if (step.optional) {
      console.log(`[fsm] step ${step.name} skipped (not present)`);
      ctx.done?.add(step.name);
      return;
    }
    throw new Error(`step ${step.name}: ready condition not met within ${timeoutMs}ms`);
  }

  const field = step.input ? await firstVisible(ctx.page, selectorsOf(step.ready) ?? []) : null;

  if (step.action) {
    await step.action(ctx);
  } else if (step.input && field) {
    const value = step.input === "username" ? ctx.creds.username : ctx.creds.password;
    await field.fill(value ?? "");
  } else if (step.input) {
    throw new Error(`step ${step.name}: input steps need string/array selectors as ready`);
  }

  if (step.submit) {
    const submit = await firstVisible(ctx.page, toArray(step.submit));
    const navigation = ctx.page
      .waitForLoadState("networkidle", { timeout: SUBMIT_SETTLE_MS })
      .catch(() => undefined);
    if (submit) await submit.click();
    else if (field) await field.press("Enter");
    else throw new Error(`step ${step.name}: no submit control`);
    await navigation;
  } else if (step.input === "password" && field) {
    await field.press("Enter");
  }

  ctx.done?.add(step.name);
}

function stepStates(steps: Step[]): Record<StateName, StateDef<LoginContext>> {
  const states: Record<StateName, StateDef<LoginContext>> = {};
  steps.forEach((step, index) => {
    const state = `STEP_${step.name}`;
    const next = step.next ?? (index + 1 < steps.length ? `STEP_${steps[index + 1].name}` : "CHECK_RESULT");
    states[state] = {
      action: (ctx) => runStep(ctx, step),
      exit: async (ctx) => ctx.done?.has(step.name) ?? false,
      next,
    };
  });
  return states;
}

function openLoginState(firstStep: StateName): StateDef<LoginContext> {
  return {
    action: async (ctx) => {
      if (ctx.page.url() !== ctx.url) {
        await ctx.page.goto(ctx.url, { waitUntil: "domcontentloaded" });
      }
      ctx.loginUrl = ctx.page.url();
    },
    exit: async () => true,
    next: firstStep,
  };
}

function sharedStates(adapter: SiteAdapter): Record<StateName, StateDef<LoginContext>> {
  return {
    CHECK_RESULT: {
      exit: async (ctx) => (await conditionMet(ctx, adapter.success)) || (await conditionMet(ctx, adapter.error)),
      next: async (ctx) => ((await conditionMet(ctx, adapter.success)) ? "AUTHENTICATED" : "LOGIN_FAILED"),
      timeoutMs: CHECK_TIMEOUT_MS,
    },

    AUTHENTICATED: {
      action: async (ctx) => {
        if (adapter.destinationUrl && ctx.page.url() !== adapter.destinationUrl) {
          await ctx.page.goto(adapter.destinationUrl, { waitUntil: "domcontentloaded" });
        }
        console.log(`[login] authenticated at ${ctx.page.url()}`);
      },
      exit: async (ctx) => {
        if (adapter.destinationUrl) return ctx.page.url() === adapter.destinationUrl;
        return conditionMet(ctx, adapter.success);
      },
      next: "AUTHENTICATED",
      terminal: true,
    },

    LOGIN_FAILED: {
      action: async (ctx) => {
        const shot = path.join(__dirname, "..", "..", "..", "login-failed.png");
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
}

export function buildLoginMachine(adapter: SiteAdapter): BuiltMachine {
  const steps = adapter.steps ?? [];
  const firstStep = steps.length ? `STEP_${steps[0].name}` : "LOGIN_PAGE";

  const states: Record<StateName, StateDef<LoginContext>> = {
    ...(steps.length ? { GOTO_LOGIN: openLoginState(firstStep) } : {}),
    ...stepStates(steps),
    ...sharedStates(adapter),
    ...(adapter.states ?? {}),
  };

  const initial = adapter.initial ?? (steps.length ? "GOTO_LOGIN" : "LOGIN_PAGE");
  return { initial, states };
}

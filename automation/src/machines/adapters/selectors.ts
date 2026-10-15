import { Locator, Page } from "playwright";
import { Condition, LoginContext, ReadyCheck } from "./types";

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function visible(page: Page, selector: string): Promise<boolean> {
  try {
    const locator = page.locator(selector).first();
    return (await locator.count()) > 0 && (await locator.isVisible());
  } catch {
    return false;
  }
}

export async function firstVisible(page: Page, selectors: string[]): Promise<Locator | null> {
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

/** Returns the selector list behind a ready check, or null for predicate checks. */
export function selectorsOf(ready: ReadyCheck): string[] | null {
  if (typeof ready === "string") return [ready];
  if (Array.isArray(ready)) return ready;
  return null;
}

export async function readyMet(ctx: LoginContext, ready: ReadyCheck): Promise<boolean> {
  if (typeof ready === "string" || Array.isArray(ready)) {
    const selectors = typeof ready === "string" ? [ready] : ready;
    return (await firstVisible(ctx.page, selectors)) !== null;
  }
  return ready(ctx);
}

export async function waitForReady(
  ctx: LoginContext,
  ready: ReadyCheck,
  timeoutMs: number,
  pollMs: number
): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await readyMet(ctx, ready)) return true;
    if (Date.now() >= deadline) return false;
    await sleep(pollMs);
  }
}

/** OR semantics across the fields set on the condition. */
export async function conditionMet(ctx: LoginContext, condition?: Condition): Promise<boolean> {
  if (!condition) return false;
  if (condition.selector && (await visible(ctx.page, condition.selector))) return true;
  if (condition.urlIncludes && ctx.page.url().includes(condition.urlIncludes)) return true;
  if (condition.matches && (await condition.matches(ctx))) return true;
  return false;
}

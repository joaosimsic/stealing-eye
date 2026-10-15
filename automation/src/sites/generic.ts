import { SiteAdapter } from "../machines/adapters/types";
import { firstVisible, visible } from "../machines/adapters/selectors";

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

/**
 * Generic fallback: guesses a single-page username+password form and clicks
 * the first submit-looking control. No site-specific selectors.
 */
export const genericAdapter: SiteAdapter = {
  id: "generic",
  initial: "LOGIN_PAGE",
  success: {
    matches: async (ctx) => {
      const onLogin = await visible(ctx.page, PASSWORD_SELECTORS[0]);
      return !onLogin && ctx.page.url() !== ctx.loginUrl;
    },
  },
  error: { selector: "[role=alert]" },
  states: {
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
  },
};

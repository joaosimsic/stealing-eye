import { StateDef, StateName } from "../../machine";
import { Condition, LoginContext, SiteAdapter } from "./types";
import { visible } from "./selectors";

export interface TwoStepOptions {
  id?: string;
  hosts?: RegExp | string[];
  destinationUrl?: string;
  /** Email/username field selectors (any-of). */
  email?: string | string[];
  /** Control that advances from the email step to the password step. */
  next?: string | string[];
  /** Password field selectors. */
  password?: string | string[];
  /** Control that submits the password step. */
  signIn?: string | string[];
  success?: Condition;
  error?: Condition;
  states?: Record<StateName, StateDef<LoginContext>>;
}

const DEFAULT_EMAIL = [
  "input[type=email]",
  "input[name*=email i]",
  "input[id*=email i]",
  "input[type=text]",
];

const DEFAULT_NEXT = ["button[type=submit]", "button"];

const DEFAULT_PASSWORD = ["input[type=password]"];

const DEFAULT_SIGN_IN = ["button[type=submit]", "button"];

/**
 * Two-step login: email (then Next) -> password (then Sign in).
 * Override any selector set per site; extra states go through `states`.
 */
export function twoStepLogin(opts: TwoStepOptions = {}): SiteAdapter {
  const success: Condition =
    opts.success ??
    {
      matches: async (ctx) =>
        !(await visible(ctx.page, "input[type=password]")) && ctx.page.url() !== ctx.loginUrl,
    };

  return {
    id: opts.id ?? "two-step",
    hosts: opts.hosts,
    destinationUrl: opts.destinationUrl,
    steps: [
      {
        name: "EMAIL",
        ready: opts.email ?? DEFAULT_EMAIL,
        input: "username",
        submit: opts.next ?? DEFAULT_NEXT,
      },
      {
        name: "PASSWORD",
        ready: opts.password ?? DEFAULT_PASSWORD,
        input: "password",
        submit: opts.signIn ?? DEFAULT_SIGN_IN,
      },
    ],
    success,
    error: opts.error ?? { selector: "[role=alert]" },
    states: opts.states,
  };
}

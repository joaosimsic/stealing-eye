import { StateDef, StateName } from "../../machine";
import { Condition, LoginContext, ReadyCheck, SearchSpec, SiteAdapter, Step } from "./types";

export interface TwoStepOptions {
  id: string;
  loginUrl: string;
  destinationUrl: string;
  /** Show search config, run after authentication. */
  search?: SearchSpec;
  /** Optional consent banner dismissed before the email step. */
  consent?: {
    ready: ReadyCheck;
    submit: string | string[];
    timeoutMs?: number;
    /** When true, a missing banner is skipped instead of failing. */
    optional?: boolean;
  };
  /** Email/username field selectors (any-of). */
  email: string | string[];
  /** Control that advances from the email step to the password step. */
  next: string | string[];
  /** Password field selectors. */
  password: string | string[];
  /** Control that submits the password step. */
  signIn: string | string[];
  success: Condition;
  error: Condition;
  states?: Record<StateName, StateDef<LoginContext>>;
}

/**
 * Two-step login: email (then Next) -> password (then Sign in).
 * Every selector, the success/error conditions and the URLs must be given
 * explicitly; there are no defaults.
 */
export function twoStepLogin(opts: TwoStepOptions): SiteAdapter {
  const steps: Step[] = [
    ...(opts.consent
      ? [
          {
            name: "CONSENT",
            ready: opts.consent.ready,
            submit: opts.consent.submit,
            timeoutMs: opts.consent.timeoutMs,
            optional: opts.consent.optional ?? false,
          },
        ]
      : []),
    {
      name: "EMAIL",
      ready: opts.email,
      input: "email" as const,
      submit: opts.next,
    },
    {
      name: "PASSWORD",
      ready: opts.password,
      input: "password" as const,
      submit: opts.signIn,
    },
  ];

  return {
    id: opts.id,
    loginUrl: opts.loginUrl,
    destinationUrl: opts.destinationUrl,
    search: opts.search,
    steps,
    success: opts.success,
    error: opts.error,
    states: opts.states,
  };
}

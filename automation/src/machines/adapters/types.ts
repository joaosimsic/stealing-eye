import { Locator, Page } from "playwright";
import { StateDef, StateName } from "../../machine";
import { Credentials } from "../../shared";

export interface LoginOptions {
  /** Force a specific adapter id instead of hostname matching. */
  site?: string;
  timeoutMs?: number;
  pollMs?: number;
}

export interface LoginContext {
  page: Page;
  creds: Credentials;
  url: string;
  opts: LoginOptions;
  adapter?: SiteAdapter;
  loginUrl?: string;
  destinationUrl?: string;
  username?: Locator | null;
  password?: Locator | null;
  submitted?: boolean;
  done?: Set<string>;
}

export interface Condition {
  /** Visible on the page. */
  selector?: string;
  /** Current URL contains this substring. */
  urlIncludes?: string;
  /** Free-form predicate for anything the above cannot express. */
  matches?: (ctx: LoginContext) => Promise<boolean>;
}

/**
 * string: single selector
 * string[]: any-of the listed selectors
 * function: custom predicate
 */
export type ReadyCheck = string | string[] | ((ctx: LoginContext) => Promise<boolean>);

export interface Step {
  /** State name becomes STEP_<NAME>. */
  name: string;
  ready: ReadyCheck;
  /** Fill username/password credential into the first visible ready selector. */
  input?: "username" | "password";
  /** Extra action; when set, it replaces the input fill. */
  action?: (ctx: LoginContext) => Promise<void>;
  /** Clicked after the input/action; Enter pressed on the input as fallback. */
  submit?: string | string[];
  timeoutMs?: number;
  /** If ready never appears: log, mark done, advance instead of failing. */
  optional?: boolean;
  next?: StateName | ((ctx: LoginContext) => StateName);
}

export interface SiteAdapter {
  id: string;
  /** Hostname matching for auto-selection. */
  hosts?: RegExp | string[];
  /** Navigated to after authentication succeeds. */
  destinationUrl?: string;
  /** Start state; defaults to first step (or LOGIN_PAGE for state-only adapters). */
  initial?: StateName;
  steps?: Step[];
  /** Escape hatch: merged last, replaces/adds states, can define whole custom FSM. */
  states?: Record<StateName, StateDef<LoginContext>>;
  success: Condition;
  error?: Condition;
}

export interface BuiltMachine {
  initial: StateName;
  states: Record<StateName, StateDef<LoginContext>>;
}

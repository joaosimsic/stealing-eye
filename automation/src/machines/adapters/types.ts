import { Locator, Page } from "playwright";
import { StateDef, StateName } from "../../machine";
import { Credentials } from "../../shared";

export interface LoginOptions {
  /** Force a specific adapter id instead of hostname matching. */
  site?: string;
  /** Show to search for after authentication. */
  show?: string;
  timeoutMs?: number;
  pollMs?: number;
}

export type Emit = (name: string, data?: Record<string, unknown>) => void;

export interface LoginContext {
  page: Page;
  creds: Credentials;
  opts: LoginOptions;
  adapter?: SiteAdapter;
  emit?: Emit;
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
  /** Fill email/password credential into the first visible ready selector. */
  input?: "email" | "password" | "show";
  /** Extra action; when set, it replaces the input fill. */
  action?: (ctx: LoginContext) => Promise<void>;
  /** Clicked after the input/action; Enter pressed on the input as fallback. */
  submit?: string | string[];
  timeoutMs?: number;
  /** If ready never appears: log, mark done, advance instead of failing. */
  optional?: boolean;
  next?: StateName | ((ctx: LoginContext) => StateName);
}

export interface SearchSpec {
  /** Optional page to open before searching; defaults to the current page. */
  url?: string;
  /** Search input selectors (any-of). */
  input: string | string[];
  /** Submit control; Enter pressed on the input when omitted. */
  submit?: string | string[];
  /** Condition that marks results as loaded. */
  expect: Condition;
  timeoutMs?: number;
}

export interface SiteAdapter {
  id: string;
  /** Entry point for login; the machine always navigates here first. */
  loginUrl: string;
  /** Navigated to after authentication succeeds. */
  destinationUrl?: string;
  /** Show search config, run after authentication. */
  search?: SearchSpec;
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

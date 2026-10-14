export type StateName = string;

export interface StateDef<C> {
  action?: (ctx: C) => Promise<void>;
  exit: (ctx: C) => Promise<boolean>;
  next: StateName | ((ctx: C) => StateName | Promise<StateName>);
  timeoutMs?: number;
  pollMs?: number;
  terminal?: boolean;
}

export interface MachineOptions<C> {
  states: Record<StateName, StateDef<C>>;
  failure: StateName;
  onTransition?: (from: StateName, to: StateName, ctx: C) => void;
  defaultTimeoutMs?: number;
  defaultPollMs?: number;
}

export class MachineError extends Error {
  constructor(
    public readonly state: StateName,
    public readonly cause: unknown
  ) {
    super(`state ${state} failed: ${cause instanceof Error ? cause.message : String(cause)}`);
    this.name = "MachineError";
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForExit<C>(
  state: StateName,
  def: StateDef<C>,
  ctx: C,
  timeoutMs: number,
  pollMs: number
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await def.exit(ctx)) return;
    if (Date.now() >= deadline) throw new MachineError(state, `exit condition not met within ${timeoutMs}ms`);
    await sleep(pollMs);
  }
}

async function resolveNext<C>(def: StateDef<C>, ctx: C): Promise<StateName> {
  return typeof def.next === "function" ? await def.next(ctx) : def.next;
}

export async function runMachine<C>(
  ctx: C,
  initial: StateName,
  opts: MachineOptions<C>
): Promise<StateName> {
  const defaultTimeoutMs = opts.defaultTimeoutMs ?? 15000;
  const defaultPollMs = opts.defaultPollMs ?? 250;
  const log = (from: StateName, to: StateName) => {
    console.log(`[fsm] ${from} -> ${to}`);
    opts.onTransition?.(from, to, ctx);
  };

  let state = initial;
  for (;;) {
    const def = opts.states[state];
    if (!def) throw new Error(`unknown state ${state}`);

    try {
      if (def.action) await def.action(ctx);
      await waitForExit(state, def, ctx, def.timeoutMs ?? defaultTimeoutMs, def.pollMs ?? defaultPollMs);
    } catch (cause) {
      if (state === opts.failure) throw cause;
      const error = cause instanceof MachineError ? cause : new MachineError(state, cause);
      console.error(`[fsm] ${error.message}`);
      log(state, opts.failure);
      state = opts.failure;
      continue;
    }

    if (def.terminal) return state;

    const next = await resolveNext(def, ctx);
    log(state, next);
    state = next;
  }
}

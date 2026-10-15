import { runMachine, StateName } from "../machine";
import { resolveAdapter } from "../sites";
import { observePage } from "../observe";
import { buildLoginMachine } from "./adapters/builder";
import { LoginContext, LoginOptions } from "./adapters/types";

export type { LoginContext, LoginOptions } from "./adapters/types";

export async function runLoginMachine(ctx: LoginContext): Promise<StateName> {
  const adapter = resolveAdapter(ctx.opts.site);
  ctx.adapter = adapter;
  ctx.done = ctx.done ?? new Set<string>();

  if (ctx.emit) observePage(ctx.page, ctx.emit);

  const machine = buildLoginMachine(adapter);
  console.log(`[login] adapter ${adapter.id}`);
  console.log(`[fsm] start ${machine.initial}`);

  const finalState = await runMachine(ctx, machine.initial, {
    states: machine.states,
    failure: "LOGIN_FAILED",
    defaultTimeoutMs: ctx.opts.timeoutMs ?? 15000,
    defaultPollMs: ctx.opts.pollMs ?? 250,
    onTransition: (from, to) => ctx.emit?.("fsm", { from, to }),
  });

  ctx.emit?.("login", {
    state: finalState === "AUTHENTICATED" ? "authenticated" : "failed",
    finalState,
  });
  return finalState;
}

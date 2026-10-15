import { runMachine, StateName } from "../machine";
import { resolveAdapter } from "../sites";
import { buildSearchMachine } from "./adapters/builder";
import { LoginContext } from "./adapters/types";

export async function runSearchMachine(ctx: LoginContext): Promise<StateName> {
  const adapter = ctx.adapter ?? resolveAdapter(ctx.opts.site);
  if (!adapter.search) throw new Error(`adapter ${adapter.id} has no search config`);
  if (!ctx.opts.show) throw new Error("SHOW must be set to search");
  ctx.done = ctx.done ?? new Set<string>();

  const machine = buildSearchMachine(adapter);
  console.log(`[search] adapter ${adapter.id} query "${ctx.opts.show}"`);
  console.log(`[fsm] start ${machine.initial}`);

  const finalState = await runMachine(ctx, machine.initial, {
    states: machine.states,
    failure: "SEARCH_NOT_FOUND",
    defaultTimeoutMs: ctx.opts.timeoutMs ?? 15000,
    defaultPollMs: ctx.opts.pollMs ?? 250,
    onTransition: (from, to) => ctx.emit?.("fsm", { from, to }),
  });

  ctx.emit?.("search", {
    state: finalState === "SEARCH_FOUND" ? "found" : "not-found",
    finalState,
  });
  return finalState;
}

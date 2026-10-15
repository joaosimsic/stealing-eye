import { SiteAdapter } from "../machines/adapters/types";
import { netflix } from "./netflix";
import { hboMax } from "./hbo-max";

export { netflix } from "./netflix";
export { hboMax } from "./hbo-max";
export { twoStepLogin } from "../machines/adapters/two-step";
export type { TwoStepOptions } from "../machines/adapters/two-step";

export const adapters: SiteAdapter[] = [netflix, hboMax];

export function resolveAdapter(siteId?: string): SiteAdapter {
  if (!siteId) {
    throw new Error(
      `no adapter selected (set SITE or --site; known: ${adapters.map((a) => a.id).join(", ")})`
    );
  }
  const found = adapters.find((a) => a.id === siteId);
  if (!found) {
    throw new Error(
      `unknown --site '${siteId}' (known: ${adapters.map((a) => a.id).join(", ")})`
    );
  }
  return found;
}

export function listAdapters(): { id: string }[] {
  return adapters.map((adapter) => ({ id: adapter.id }));
}

import { SiteAdapter } from "../machines/adapters/types";
import { genericAdapter } from "./generic";
import { netflix } from "./netflix";
import { twoStep } from "./two-step";

export { genericAdapter } from "./generic";
export { netflix } from "./netflix";
export { twoStep } from "./two-step";
export { twoStepLogin } from "../machines/adapters/two-step";
export type { TwoStepOptions } from "../machines/adapters/two-step";

export const adapters: SiteAdapter[] = [genericAdapter, netflix, twoStep];

function hostMatches(adapter: SiteAdapter, hostname: string): boolean {
  if (!adapter.hosts || !hostname) return false;
  const list = Array.isArray(adapter.hosts) ? adapter.hosts : [adapter.hosts];
  return list.some((host) =>
    typeof host === "string" ? hostname === host || hostname.endsWith(`.${host}`) : host.test(hostname)
  );
}

export function resolveAdapter(url: string, override?: string): SiteAdapter {
  if (override) {
    const found = adapters.find((a) => a.id === override);
    if (!found) {
      throw new Error(
        `unknown --site '${override}' (known: ${adapters.map((a) => a.id).join(", ")})`
      );
    }
    return found;
  }
  let hostname = "";
  try {
    hostname = new URL(url).hostname;
  } catch {
    // fall through to generic
  }
  return adapters.find((a) => hostMatches(a, hostname)) ?? genericAdapter;
}

export function listAdapters(): { id: string; hosts?: string }[] {
  return adapters.map((adapter) => ({
    id: adapter.id,
    hosts: adapter.hosts
      ? Array.isArray(adapter.hosts)
        ? adapter.hosts.join(",")
        : adapter.hosts.source
      : undefined,
  }));
}

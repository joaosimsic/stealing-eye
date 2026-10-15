import { LoginContext } from "./machines/adapters/types";
import { runLoginMachine } from "./machines/login";
import { runSearchMachine } from "./machines/search";

/** Full automation: authenticate, then search for the configured show. */
export async function runAutomation(ctx: LoginContext): Promise<void> {
  const login = await runLoginMachine(ctx);
  if (login !== "AUTHENTICATED") {
    throw new Error(`login ended in ${login}`);
  }

  const search = await runSearchMachine(ctx);
  if (search !== "SEARCH_FOUND") {
    throw new Error(`search ended in ${search}`);
  }
}

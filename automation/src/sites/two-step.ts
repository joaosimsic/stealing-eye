import { twoStepLogin } from "../machines/adapters/two-step";

/** Host-agnostic two-step template; opt in with `--site two-step`. */
export const twoStep = twoStepLogin();

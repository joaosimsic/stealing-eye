#!/usr/bin/env node
import { envCredentials, envLoginOptions, launchBrowser } from "./shared";
import { runLoginMachine } from "./machines/login";

function parseArgs(argv: string[]): { url: string | null } {
  let url: string | null = null;
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith("-")) throw new Error(`unknown flag ${a}`);
    else url = a;
  }
  return { url };
}

async function main(): Promise<void> {
  const creds = envCredentials();
  const { url: argUrl } = parseArgs(process.argv);
  const url = argUrl || creds.url;
  if (!url) throw new Error("no URL given (pass as arg or set URL in .env)");
  if (!creds.username || !creds.password) {
    throw new Error("USERNAME and PASSWORD must be set in .env to log in");
  }

  const browser = await launchBrowser();
  const context = await browser.newContext();
  const page = await context.newPage();

  const finalState = await runLoginMachine({ page, creds, url, opts: envLoginOptions() });
  if (finalState === "AUTHENTICATED") {
    console.log("logged in; browser stays open (Ctrl-C to quit)");
  } else {
    console.error("login failed; browser stays open for inspection (Ctrl-C to quit)");
  }

  const shutdown = () => {
    browser.close().finally(() => process.exit(0));
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  await new Promise<void>(() => {});
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

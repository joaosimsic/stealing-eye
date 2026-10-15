#!/usr/bin/env node
import { envCredentials, envLoginOptions, launchBrowser } from "./shared";
import { runLoginMachine } from "./machines/login";
import { listAdapters } from "./sites";

interface Args {
  site?: string;
  list: boolean;
}

function parseArgs(argv: string[]): Args {
  const args: Args = { list: false };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--site") args.site = argv[++i];
    else if (a === "--list-adapters") args.list = true;
    else throw new Error(`unknown flag ${a}`);
  }
  return args;
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv);

  if (args.list) {
    for (const adapter of listAdapters()) {
      console.log(adapter.id);
    }
    return;
  }

  const creds = envCredentials();
  if (!creds.email || !creds.password) {
    throw new Error("EMAIL and PASSWORD must be set in .env to log in");
  }

  const browser = await launchBrowser();
  const context = await browser.newContext();
  const page = await context.newPage();

  const opts = { ...envLoginOptions(), ...(args.site ? { site: args.site } : {}) };
  const emit = (name: string, data?: Record<string, unknown>): void => {
    console.log(`[event] ${name} ${data ? JSON.stringify(data) : ""}`.trim());
  };
  const finalState = await runLoginMachine({ page, creds, opts, emit });
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

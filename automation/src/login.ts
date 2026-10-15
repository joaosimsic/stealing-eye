#!/usr/bin/env node
import { envCredentials, envLoginOptions, launchBrowser, validateEnv } from "./shared";
import { runAutomation } from "./run";
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

  validateEnv();
  const creds = envCredentials();

  const browser = await launchBrowser();
  const context = await browser.newContext();
  const page = await context.newPage();

  const opts = { ...envLoginOptions(), ...(args.site ? { site: args.site } : {}) };
  const emit = (name: string, data?: Record<string, unknown>): void => {
    console.log(`[event] ${name} ${data ? JSON.stringify(data) : ""}`.trim());
  };

  let exitCode = 0;
  try {
    await runAutomation({ page, creds, opts, emit });
    console.log("authenticated and searched; browser stays open (Ctrl-C to quit)");
  } catch (e) {
    exitCode = 1;
    console.error((e as Error).message);
    console.error("automation failed; browser stays open for inspection (Ctrl-C to quit)");
  }

  const shutdown = () => {
    browser.close().finally(() => process.exit(exitCode));
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  await new Promise<void>(() => {});
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

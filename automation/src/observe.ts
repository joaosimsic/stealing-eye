import { Page } from "playwright";
import { Emit } from "./machines/adapters/types";

/** Emits page lifecycle events so loading vs. failure is visible in the event stream. */
export function observePage(page: Page, emit: Emit): void {
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) emit("page", { phase: "navigate", url: frame.url() });
  });
  page.on("domcontentloaded", () => emit("page", { phase: "dom", url: page.url() }));
  page.on("load", () => emit("page", { phase: "load", url: page.url() }));
  page.on("requestfailed", (request) =>
    emit("page", {
      phase: "request-failed",
      url: request.url(),
      error: request.failure()?.errorText,
    })
  );
  page.on("response", (response) => {
    if (response.status() >= 400) {
      emit("page", { phase: "http", status: response.status(), url: response.url() });
    }
  });
  page.on("crash", () => emit("page", { phase: "crash" }));
}

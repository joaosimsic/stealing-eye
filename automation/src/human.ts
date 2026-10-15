import { Locator } from "playwright";

const TYPE_MS: [number, number] = [60, 180];
const ACTION_MS: [number, number] = [300, 900];
const CLICK_MS: [number, number] = [20, 80];
const MOVE_STEPS: [number, number] = [8, 20];
const JITTER_PX = 5;

export function rand([min, max]: [number, number]): number {
  return min + Math.random() * (max - min);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function pauseAction(): Promise<void> {
  return sleep(rand(ACTION_MS));
}

/** Types text one key at a time with a random per-keystroke delay. */
export async function typeHuman(locator: Locator, text: string): Promise<void> {
  await locator.selectText();
  for (const char of text) {
    await locator.pressSequentially(char, { delay: rand(TYPE_MS) });
  }
}

/** Moves the pointer to the element in a randomized path, then clicks. */
export async function clickHuman(locator: Locator): Promise<void> {
  const page = locator.page();
  const box = await locator.boundingBox();
  if (box) {
    const x = box.x + box.width / 2 + rand([-JITTER_PX, JITTER_PX]);
    const y = box.y + box.height / 2 + rand([-JITTER_PX, JITTER_PX]);
    await page.mouse.move(x, y, { steps: Math.round(rand(MOVE_STEPS)) });
  }
  await pauseAction();
  await locator.click({ delay: rand(CLICK_MS) });
}

/** Waits, then presses a key (e.g. Enter fallback). */
export async function pressHuman(locator: Locator, key: string): Promise<void> {
  await pauseAction();
  await locator.press(key);
}

import { twoStepLogin } from "../machines/adapters/two-step";

export const netflix = twoStepLogin({
  id: "netflix",
  hosts: ["netflix.com"],
  email: ['input[name="userLoginId"]', 'input[type="email"]', 'input[type="text"]'],
  next: ['button[data-uia="continue-button"]', 'button[type="submit"]'],
  password: ['input[name="password"]', 'input[type="password"]'],
  signIn: ['button[data-uia="continue-button"]', 'button[type="submit"]'],
  success: { urlIncludes: "/browse" },
  error: { selector: '[data-uia="error-message-container"], .inputError' },
});

import { twoStepLogin } from "../machines/adapters/two-step";

export const hboMax = twoStepLogin({
  id: "hbo-max",
  loginUrl: "https://auth.hbomax.com/login",
  destinationUrl: "https://play.hbomax.com/",
  search: {
    input: ['input[type="search"]'],
    submit: ['button[type="submit"]'],
    expect: { selector: '[data-testid="search-results"]' },
  },
  consent: {
    ready: '[data-testid="consent-modal-button-0"]',
    submit: '[data-testid="consent-modal-button-0"]',
  },
  email: ['input[id="sign-in-phoneEmail-input"]'],
  next: ['button[type="submit"]'],
  password: ['input[id="sign-in-password-password-input"]'],
  signIn: ['button[type="submit"]'],
  success: { urlIncludes: "profile-picker" },
  error: {
    selector:
      '.notification-banner-backplate-error, [data-testid*=".error." i]',
  },
});

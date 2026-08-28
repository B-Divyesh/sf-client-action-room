import { PublicClientApplication, type AccountInfo } from '@azure/msal-browser';

const clientId = '25c704f4-465a-47af-80ab-2c489466b697';
const authority = 'https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/';
const scopes = ['openid', 'profile', 'email'];

const client = new PublicClientApplication({
  auth: {
    clientId,
    authority,
    redirectUri: `${window.location.origin}/auth/callback`,
    postLogoutRedirectUri: window.location.origin,
  },
  cache: { cacheLocation: 'sessionStorage' },
});

let initialized = false;

async function ready() {
  if (!initialized) {
    await client.initialize();
    initialized = true;
  }
  return client;
}

export async function beginStaffSignIn() {
  const app = await ready();
  await app.loginRedirect({ scopes, redirectStartPage: `${window.location.origin}/workspace` });
}

export async function finishStaffSignIn(): Promise<AccountInfo | null> {
  const app = await ready();
  const result = await app.handleRedirectPromise();
  return result?.account ?? app.getAllAccounts()[0] ?? null;
}

export async function staffToken(account: AccountInfo): Promise<string> {
  const app = await ready();
  const result = await app.acquireTokenSilent({ account, scopes });
  return result.idToken;
}

export async function signOut(account: AccountInfo) {
  const app = await ready();
  await app.logoutRedirect({ account });
}

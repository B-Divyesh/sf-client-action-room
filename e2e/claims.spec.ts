import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test';

async function freshContext(browser: Browser, mobile = false): Promise<BrowserContext> {
  return browser.newContext({
    viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 900 },
  });
}

async function openDemo(browser: Browser, mobile = false) {
  const context = await freshContext(browser, mobile);
  const page = await context.newPage();
  await page.goto('/demo');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Your sample client action room');
  return { context, page };
}

async function publishAndApprove(page: Page, context: BrowserContext) {
  await page.getByRole('button', { name: 'Publish client link' }).click();
  const link = await page.getByTestId('open-client-link').getAttribute('href');
  expect(link).toContain('/client#access=');
  const clientPage = await context.newPage();
  await clientPage.goto(link!);
  await expect(clientPage.getByRole('heading', { level: 1 })).toHaveText(
    'Approve the final menu proof',
  );
  await clientPage.getByLabel('Approve this request').check();
  await clientPage.getByRole('button', { name: 'Record my answer' }).click();
  await expect(clientPage.getByTestId('client-completion')).toContainText('Approval recorded');
  return clientPage;
}

test('@claim:demo-one-click Try a ready client action room in one click', async ({ browser }) => {
  const context = await freshContext(browser);
  const page = await context.newPage();
  await page.goto('/');
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Your sample client action room');
  await expect(page.getByText('Alder Street Bakery launch').first()).toBeVisible();
  await expect(page.locator('.action-slip')).toHaveCount(4);
  await context.close();
});

test('@claim:demo-reset Demo changes are sample-only and resettable', async ({ browser }) => {
  const first = await openDemo(browser);
  const second = await openDemo(browser);
  const clientPage = await publishAndApprove(first.page, first.context);
  await first.page.reload();
  await expect(first.page.locator('[data-event="client_decision_recorded"]')).toContainText('Maya Chen');

  await second.page.reload();
  const secondApproval = second.page.locator('.action-slip[data-kind="approval"]');
  await expect(secondApproval).not.toContainText('Complete');
  await expect(second.page.locator('[data-event="client_decision_recorded"]')).toHaveCount(0);

  await first.page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(first.page.getByText('The demo is back to its original sample.')).toBeVisible();
  await expect(first.page.locator('[data-event="client_decision_recorded"]')).toHaveCount(0);
  await expect(first.page.locator('.action-slip')).toHaveCount(4);
  await clientPage.close();
  await first.context.close();
  await second.context.close();
});

test('@claim:client-no-account A client can approve a request without creating an account', async ({ browser }) => {
  const { context, page } = await openDemo(browser, true);
  const clientPage = await publishAndApprove(page, context);
  await expect(clientPage).toHaveURL(/\/client$/);
  await expect(clientPage.locator('input[type="password"]')).toHaveCount(0);
  await expect(clientPage.getByText('Your answer is recorded.')).toBeVisible();
  await expect(clientPage.getByText('Maya Chen ·')).toBeVisible();
  await context.close();
});

test('@claim:deadline-order Open requests are ordered by deadline', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const slips = page.locator('.action-slip');
  const dueValues = await slips.evaluateAll((nodes) => nodes.map((node) => node.getAttribute('data-due')!));
  const sorted = [...dueValues].sort((left, right) => Date.parse(left) - Date.parse(right));
  expect(dueValues).toEqual(sorted);
  await expect(slips.first()).toContainText('Overdue');
  await expect(slips.first()).toContainText('Upload the signed allergen sheet');
  await context.close();
});

test('@claim:approval-audit An approval records the decision, actor label, and server time', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const submittedAfter = Date.now();
  await publishAndApprove(page, context);
  await page.reload();
  const event = page.locator('[data-event="client_decision_recorded"]');
  await expect(event).toContainText('Approval recorded');
  await expect(event).toContainText('Maya Chen · Approved');
  const recordedAt = await event.locator('time').getAttribute('datetime');
  if (process.env.PLAYWRIGHT_BASE_URL) {
    expect(Date.parse(recordedAt!)).toBeGreaterThanOrEqual(submittedAfter - 1_000);
    expect(Date.parse(recordedAt!)).toBeLessThanOrEqual(Date.now() + 1_000);
  } else {
    expect(recordedAt).toBe('2026-08-28T14:00:00+00:00');
  }
  await context.close();
});

test('@claim:link-expiry Client links last seven days, then cannot read or submit the request', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const queueResponse = await context.request.get('/api/v1/demo/queue');
  const queue = await queueResponse.json();
  const approvalId = queue.actions.find((action: { kind: string }) => action.kind === 'approval').id;
  const published = await context.request.post(`/api/v1/demo/actions/${approvalId}/publish`);
  const publishedBody = await published.json();
  expect(Date.parse(publishedBody.expires_at) - Date.parse(queue.server_now)).toBe(7 * 24 * 60 * 60 * 1000);
  await page.getByRole('button', { name: 'Create expired link example' }).click();
  const href = await page.getByTestId('expired-client-link').getAttribute('href');
  const expiredPage = await context.newPage();
  await expiredPage.goto(href!);
  await expect(expiredPage.getByRole('heading', { level: 1 })).toHaveText(
    'This client link has expired',
  );
  await expect(expiredPage.getByText('No request or workspace details were shown.')).toBeVisible();
  await expect(expiredPage.getByText('Approve the final menu proof')).toHaveCount(0);
  const direct = await context.request.post('/api/v1/client/actions/not-available/submissions', {
    data: { actor_label: 'Maya Chen', decision: 'approved', comment: '' },
    headers: { 'Idempotency-Key': 'expired-direct-submit' },
  });
  expect(direct.status()).toBe(401);
  await context.close();
});

async function openTypedAction(page: Page, context: BrowserContext, kind: 'upload' | 'choice' | 'external') {
  await page.locator(`.action-slip[data-kind="${kind === 'external' ? 'external_link' : kind}"]`).getByRole('button', { name: `Open ${kind} request` }).click();
  const href = await page.getByTestId('open-client-link').getAttribute('href');
  const clientPage = await context.newPage();
  await clientPage.goto(href!);
  return clientPage;
}

test('@claim:secure-upload A client PDF is type-checked, malware-scanned, and scoped', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const clientPage = await openTypedAction(page, context, 'upload');
  await expect(clientPage.getByRole('heading', { level: 1 })).toHaveText('Upload the signed allergen sheet');
  const queueResponse = await context.request.get('/api/v1/demo/queue');
  const queue = await queueResponse.json();
  const choiceId = queue.actions.find((action: { kind: string }) => action.kind === 'choice').id;
  const outsideScope = await context.request.post(`/api/v1/client/actions/${choiceId}/choice`, {
    data: { actor_label: 'Maya Chen', option_key: 'square' },
  });
  expect(outsideScope.status()).toBe(403);
  await clientPage.getByLabel('Signed sheet (PDF, up to 5 MB)').setInputFiles({
    name: 'notes.txt', mimeType: 'text/plain', buffer: Buffer.from('not a PDF'),
  });
  await clientPage.getByRole('button', { name: 'Upload and scan file' }).click();
  await expect(clientPage.getByRole('alert')).toContainText('Upload one PDF file under 5 MB');
  await clientPage.getByLabel('Signed sheet (PDF, up to 5 MB)').setInputFiles({
    name: 'unsafe.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4\nX5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*'),
  });
  await clientPage.getByRole('button', { name: 'Upload and scan file' }).click();
  await expect(clientPage.getByRole('alert')).toContainText('malware scan rejected', { timeout: 20_000 });
  await clientPage.getByLabel('Signed sheet (PDF, up to 5 MB)').setInputFiles({
    name: 'signed-allergen-sheet.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF'),
  });
  await clientPage.getByRole('button', { name: 'Upload and scan file' }).click();
  await expect(clientPage.getByTestId('client-completion')).toContainText('File received and malware-scanned', { timeout: 20_000 });
  await page.reload();
  await expect(page.locator('[data-event="client_file_scanned"]')).toContainText('Clean PDF');
  await context.close();
});

test('@claim:choice-flow A client can choose one listed option through a scoped link', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const clientPage = await openTypedAction(page, context, 'choice');
  await clientPage.getByLabel('Square pastry crop').check();
  await clientPage.getByRole('button', { name: 'Record my choice' }).click();
  await expect(clientPage.getByTestId('client-completion')).toContainText('Square pastry crop');
  await page.reload();
  await expect(page.locator('[data-event="client_choice_recorded"]')).toContainText('Square pastry crop');
  await context.close();
});

test('@claim:external-link A client sees the destination before opening an HTTPS payment or booking link', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const clientPage = await openTypedAction(page, context, 'external');
  await expect(clientPage.getByText('The destination is')).toContainText('example.com');
  await clientPage.getByRole('button', { name: 'Open example.com' }).click();
  const outbound = clientPage.getByRole('link', { name: 'Continue to example.com' });
  await expect(outbound).toHaveAttribute('href', 'https://example.com/');
  await page.reload();
  await expect(page.locator('[data-event="external_link_opened"]')).toContainText('example.com');
  await context.close();
});

test('@claim:reminder-audit Staff can schedule one reminder and see its audit record', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const approval = page.locator('.action-slip[data-kind="approval"]');
  await approval.getByRole('button', { name: 'Schedule reminder' }).click();
  await expect(page.getByRole('status')).toContainText('Reminder scheduled');
  await expect(page.locator('[data-event="reminder_scheduled"]')).toContainText('Theo Grant');
  await context.close();
});

test('@claim:real-workspace A firm starts with an empty, isolated workspace that persists', async ({ browser }) => {
  test.skip(Boolean(process.env.PLAYWRIGHT_BASE_URL), 'Test identities are disabled outside the local sandbox.');
  const owner = await freshContext(browser);
  const ownerId = `firm-owner-${Date.now()}`;
  const ownerHeaders = { Authorization: `Bearer test:${ownerId}` };
  const initial = await owner.request.get('/api/v1/staff/workspace', { headers: ownerHeaders });
  expect(initial.status()).toBe(404);

  const created = await owner.request.post('/api/v1/staff/workspace', {
    headers: ownerHeaders,
    data: {
      firm_name: 'River & Pine',
      client_label: 'March launch',
      client_actor: 'Ari Kim',
    },
  });
  expect(created.status()).toBe(201);
  expect((await created.json()).actions).toEqual([]);

  const action = await owner.request.post('/api/v1/staff/actions', {
    headers: { ...ownerHeaders, 'Idempotency-Key': 'real-action-create' },
    data: {
      title: 'Approve the launch copy',
      instructions: 'Check the three headings and record your answer.',
      due_at: '2026-08-29T14:00:00Z',
    },
  });
  expect(action.status()).toBe(201);
  const actionId = (await action.json()).id;
  const published = await owner.request.post(`/api/v1/staff/actions/${actionId}/publish`, {
    headers: { ...ownerHeaders, 'Idempotency-Key': 'real-action-publish' },
  });
  expect(published.status()).toBe(200);
  const link = (await published.json()).path as string;

  const client = await freshContext(browser);
  const clientPage = await client.newPage();
  await clientPage.goto(link);
  await expect(clientPage.getByRole('heading', { level: 1 })).toHaveText('Approve the launch copy');
  await expect(clientPage.getByText('River & Pine', { exact: true })).toBeVisible();
  await clientPage.getByLabel('Approve this request').check();
  await clientPage.getByRole('button', { name: 'Record my answer' }).click();
  await expect(clientPage.getByTestId('client-completion')).toContainText('Approval recorded');

  const persisted = await owner.request.get('/api/v1/staff/workspace', { headers: ownerHeaders });
  const persistedBody = await persisted.json();
  expect(persistedBody.namespace).toBe('real');
  expect(persistedBody.actions).toHaveLength(1);
  expect(persistedBody.actions[0].status).toBe('completed');
  expect(persistedBody.audit.some((event: { event_name: string }) => event.event_name === 'client_decision_recorded')).toBe(true);

  const other = await freshContext(browser);
  const denied = await other.request.get('/api/v1/staff/workspace', {
    headers: { Authorization: `Bearer test:other-${ownerId}` },
  });
  expect(denied.status()).toBe(404);
  await client.close();
  await owner.close();
  await other.close();
});

test('@claim:demo-privacy Demo traffic stays on this site and leaving deletes the room', async ({ browser }) => {
  const context = await freshContext(browser);
  const page = await context.newPage();
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/demo');
  const queueBody = await page.evaluate(async () => {
    const response = await fetch('/api/v1/demo/queue');
    if (!response.ok) throw new Error(`Queue request failed with ${response.status}`);
    return response.json();
  });
  const lifetimeSeconds = (Date.parse(queueBody.expires_at) - Date.parse(queueBody.server_now)) / 1000;
  expect(lifetimeSeconds).toBeGreaterThan(0);
  expect(lifetimeSeconds).toBeLessThanOrEqual(86_400);

  const approval = page.locator('.action-slip[data-kind="approval"]');
  await approval.getByRole('button', { name: 'Schedule reminder' }).click();

  await page.getByRole('button', { name: 'Publish client link' }).click();
  const link = await page.getByTestId('open-client-link').getAttribute('href');
  expect(link).toContain('#access=');
  const clientPage = await context.newPage();
  const clientRequests: string[] = [];
  clientPage.on('request', (request) => clientRequests.push(request.url()));
  await clientPage.goto(link!);
  await expect(clientPage).toHaveURL(/\/client$/);
  expect(clientRequests.some((url) => url.includes('access='))).toBe(false);

  const removed = await context.request.delete('/api/v1/demo/session');
  expect(removed.status()).toBe(204);
  const gone = await context.request.get('/api/v1/demo/queue');
  expect([401, 410]).toContain(gone.status());
  const expectedOrigin = new URL(process.env.PLAYWRIGHT_BASE_URL ?? 'http://127.0.0.1:4173').origin;
  expect([...requests, ...clientRequests].every((url) => new URL(url).origin === expectedOrigin)).toBe(true);
  await context.close();
});

test('@claim:staff-auth Staff access rejects missing or invalid tokens', async ({ browser }) => {
  const context = await freshContext(browser);
  const page = await context.newPage();
  await page.goto('/workspace');
  await expect(page.getByRole('button', { name: 'Sign in with Sociobot' })).toBeVisible();
  await expect(page.locator('input[type="password"]')).toHaveCount(0);
  const missing = await context.request.get('/api/v1/staff/workspace');
  expect(missing.status()).toBe(401);
  expect(missing.headers()['www-authenticate']).toBe('Bearer');
  const invalid = await context.request.get('/api/v1/staff/workspace', {
    headers: { Authorization: 'Bearer not-a-jwt' },
  });
  expect(invalid.status()).toBe(401);
  expect(invalid.headers()['www-authenticate']).toBe('Bearer');
  await context.close();
});

test('mobile, keyboard, routing, accessibility, and request privacy smoke', async ({ browser }) => {
  const context = await freshContext(browser, true);
  const page = await context.newPage();
  const consoleErrors: string[] = [];
  const outsideRequests: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });
  page.on('pageerror', (error) => consoleErrors.push(error.message));
  page.on('request', (request) => {
    const expectedOrigin = new URL(process.env.PLAYWRIGHT_BASE_URL ?? 'http://127.0.0.1:4173').origin;
    if (new URL(request.url()).origin !== expectedOrigin) outsideRequests.push(request.url());
  });
  await page.goto('/demo');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page).toHaveTitle('Demo — Client Action Room');
  await expect(page.locator('h1')).toBeFocused();
  await page.getByRole('link', { name: 'Skip to main content' }).focus();
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical')).toEqual([]);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  expect(outsideRequests).toEqual([]);
  expect(consoleErrors).toEqual([]);

  await page.goto('/privacy');
  await expect(page).toHaveTitle('Privacy — Client Action Room');
  await page.goto('/missing-record');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('We could not find this page');
  await page.goBack();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('How Client Action Room handles data');
  await context.close();
});

test('staff workspace uses the CIAM boundary and remains keyboard accessible', async ({ browser }) => {
  const context = await freshContext(browser, true);
  const page = await context.newPage();
  await page.goto('/workspace');
  await expect(page).toHaveTitle('Workspace — Client Action Room');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Open your firm workspace');
  const signIn = page.getByRole('button', { name: 'Sign in with Sociobot' });
  await signIn.focus();
  await expect(signIn).toBeFocused();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter(({ impact }) => impact === 'serious' || impact === 'critical')).toEqual([]);

  const unauthorized = await context.request.get('/api/v1/me');
  expect(unauthorized.status()).toBe(401);
  expect(unauthorized.headers()['www-authenticate']).toBe('Bearer');
  await context.close();
});

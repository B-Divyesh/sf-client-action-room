import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test';

let addressCounter = 20;

async function freshContext(browser: Browser, mobile = false): Promise<BrowserContext> {
  addressCounter += 1;
  return browser.newContext({
    viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 900 },
    extraHTTPHeaders: { 'X-Forwarded-For': `198.51.100.${addressCounter}` },
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
  await clientPage.getByLabel('Approve this proof').check();
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

test('@claim:link-expiry An expired client link cannot read or submit the request', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
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

test('@claim:secure-upload A client PDF is type-checked, safety-scanned, and scoped', async ({ browser }) => {
  const { context, page } = await openDemo(browser);
  const clientPage = await openTypedAction(page, context, 'upload');
  await expect(clientPage.getByRole('heading', { level: 1 })).toHaveText('Upload the signed allergen sheet');
  await clientPage.getByLabel('Signed sheet (PDF, up to 5 MB)').setInputFiles({
    name: 'unsafe.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4 EICAR test marker'),
  });
  await clientPage.getByRole('button', { name: 'Upload and scan file' }).click();
  await expect(clientPage.getByRole('alert')).toContainText('safety scan rejected');
  await clientPage.getByLabel('Signed sheet (PDF, up to 5 MB)').setInputFiles({
    name: 'signed-allergen-sheet.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF'),
  });
  await clientPage.getByRole('button', { name: 'Upload and scan file' }).click();
  await expect(clientPage.getByTestId('client-completion')).toContainText('File received and scanned');
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
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('This record is not in the archive');
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

import { defineConfig, devices } from '@playwright/test';

const externalBaseURL = process.env.PLAYWRIGHT_BASE_URL;
const baseURL = externalBaseURL ?? 'http://127.0.0.1:4173';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  timeout: 30_000,
  expect: { timeout: 8_000 },
  reporter: [['list'], ['html', { open: 'never', outputFolder: '.factory/evidence/playwright-report' }]],
  outputDir: '.factory/evidence/test-results',
  use: {
    baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    ...devices['Desktop Chrome'],
  },
  webServer: externalBaseURL
    ? undefined
    : {
        command:
          'npm run build:web && DATA_DIR=.data-test DIST_DIR=dist PORT=4173 DEMO_FIXED_NOW=2026-08-28T14:00:00Z MALWARE_SCANNER_MODE=fixture AUTH_TEST_MODE=1 cargo run --manifest-path server/Cargo.toml',
        url: 'http://127.0.0.1:4173/health',
        reuseExistingServer: false,
        timeout: 120_000,
      },
});

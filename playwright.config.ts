import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  timeout: 30_000,
  expect: { timeout: 8_000 },
  reporter: [['list'], ['html', { open: 'never', outputFolder: '.factory/evidence/playwright-report' }]],
  outputDir: '.factory/evidence/test-results',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    ...devices['Desktop Chrome'],
  },
  webServer: {
    command:
      'npm run build:web && DATA_DIR=.data-test DIST_DIR=dist PORT=4173 DEMO_FIXED_NOW=2026-08-28T14:00:00Z cargo run --manifest-path server/Cargo.toml',
    url: 'http://127.0.0.1:4173/health',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});

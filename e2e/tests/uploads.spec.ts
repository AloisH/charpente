import { expect, test } from "@playwright/test";

import { register, resetDb } from "./helpers";

test.beforeEach(async ({ request }) => {
  await resetDb(request);
});

test("file upload round-trip: declare → PUT to storage → confirm → list", async ({ page }) => {
  await register(page, {
    email: "dave@example.com",
    password: "correct-horse-battery",
    displayName: "Dave",
  });

  await page.getByRole("link", { name: /files|fichiers/i }).click();
  await page.waitForURL("**/uploads");

  await page.locator('input[type="file"]').setInputFiles({
    name: "hello.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("hello from the e2e suite"),
  });

  // The file appears in the list once the presigned flow completes.
  await expect(page.getByText("hello.txt")).toBeVisible({ timeout: 15_000 });
  await expect(page.getByRole("button", { name: /download|télécharger/i })).toBeVisible();
});

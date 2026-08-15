import { expect, test } from "@playwright/test";

import { ADMIN, login, logout, register, resetDb } from "./helpers";

test.beforeEach(async ({ request }) => {
  await resetDb(request);
});

test("admin impersonates a user and steps back out", async ({ page }) => {
  await register(page, {
    email: "victim@example.com",
    password: "correct-horse-battery",
    displayName: "Victor",
  });
  await logout(page);

  await login(page, ADMIN);
  await page.goto("/admin/users");

  // Impersonate Victor from his row.
  await page
    .getByRole("row", { name: /victim@example\.com/ })
    .getByRole("button", { name: /impersonate|incarner/i })
    .click();
  await page.waitForURL("**/dashboard");

  // The unmissable banner, naming the impersonated user.
  const banner = page.getByRole("alert").filter({ hasText: /victor/i });
  await expect(banner).toBeVisible();

  // One click out: back on the admin page, banner gone, admin session again.
  await banner.getByRole("button", { name: /stop impersonating|arrêter/i }).click();
  await page.waitForURL("**/admin/users");
  await expect(page.getByRole("alert").filter({ hasText: /victor/i })).toHaveCount(0);
});

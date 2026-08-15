import { expect, test } from "@playwright/test";

import { ADMIN, login, register, resetDb } from "./helpers";

test.beforeEach(async ({ request }) => {
  await resetDb(request);
});

test("admin sees the users page; plain users are bounced", async ({ page }) => {
  // Plain user: no admin nav, direct access redirects home.
  await register(page, {
    email: "carol@example.com",
    password: "correct-horse-battery",
    displayName: "Carol",
  });
  await expect(page.getByRole("link", { name: /admin/i })).toHaveCount(0);
  await page.goto("/admin/users");
  await page.waitForURL((url) => !url.pathname.startsWith("/admin"));

  await page.getByRole("button", { name: /log out|se déconnecter/i }).click();
  await page.waitForURL("**/login");

  // Seeded admin: link visible, page lists both accounts.
  await login(page, ADMIN);
  await page.getByRole("link", { name: /admin/i }).click();
  await page.waitForURL("**/admin/users");
  await expect(page.getByText("carol@example.com")).toBeVisible();
  await expect(page.getByText(ADMIN.email)).toBeVisible();
});

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import { login, register, resetDb } from "./helpers";

test.beforeEach(async ({ request }) => {
  await resetDb(request);
});

test("signup → protected page → logout → login", async ({ page }) => {
  await register(page, {
    email: "alice@example.com",
    password: "correct-horse-battery",
    displayName: "Alice",
  });

  // Landed on the protected dashboard, greeted by name.
  await expect(page.getByText(/alice/i).first()).toBeVisible();

  // Logout bounces back to login.
  await page.getByRole("button", { name: /log out|se déconnecter/i }).click();
  await page.waitForURL("**/login");

  // The protected page now redirects to login.
  await page.goto("/dashboard");
  await page.waitForURL(/login/);

  // And logging back in works.
  await login(page, { email: "alice@example.com", password: "correct-horse-battery" });
});

test("login rejects wrong password", async ({ page }) => {
  await register(page, {
    email: "bob@example.com",
    password: "correct-horse-battery",
    displayName: "Bob",
  });
  await page.getByRole("button", { name: /log out|se déconnecter/i }).click();
  await page.waitForURL("**/login");

  await page.getByLabel(/e-?mail/i).fill("bob@example.com");
  await page.getByLabel(/password|mot de passe/i).fill("wrong-password");
  await page.getByRole("button", { name: /log in|se connecter/i }).click();

  await expect(page.getByText(/invalid|invalide/i).first()).toBeVisible();
  expect(page.url()).toContain("/login");
});

test("login page has no serious accessibility violations", async ({ page }) => {
  await page.goto("/login");
  const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
  const serious = results.violations.filter(
    (violation) => violation.impact === "serious" || violation.impact === "critical",
  );
  expect(serious).toEqual([]);
});

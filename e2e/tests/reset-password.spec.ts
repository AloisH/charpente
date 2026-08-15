import { expect, test } from "@playwright/test";

import { clearMailbox, login, logout, mailBody, register, resetDb } from "./helpers";

test.beforeEach(async ({ request }) => {
  await resetDb(request);
  await clearMailbox(request);
});

test("password reset: forgot → mail arrives → new password → old one dead", async ({
  page,
  request,
}) => {
  await register(page, {
    email: "oubli@example.com",
    password: "original-password",
    displayName: "Oubli",
  });
  await logout(page);
  // Only the reset mail should be in the box when we search below.
  await clearMailbox(request);

  await page.goto("/login");
  await page.getByRole("link", { name: /forgot password|mot de passe oublié/i }).click();
  await page.waitForURL("**/forgot-password");
  await page.getByLabel(/e-?mail/i).fill("oubli@example.com");
  await page.getByRole("button", { name: /send reset link|envoyer le lien/i }).click();
  await expect(page.getByText(/reset link is on its way|est en route/i)).toBeVisible();

  const body = await mailBody(request, "oubli@example.com");
  const link = body.match(/http:\/\/localhost:8180\/reset-password\?token=[\w-]+/)?.[0];
  expect(link).toBeDefined();

  await page.goto(link as string);
  await page.getByLabel(/new password|nouveau mot de passe/i).fill("fresh-new-password");
  await page.getByRole("button", { name: /change password|changer le mot de passe/i }).click();
  await page.waitForURL("**/login");

  // Old password rejected…
  await page.getByLabel(/e-?mail/i).fill("oubli@example.com");
  await page.getByLabel(/^password|^mot de passe/i).fill("original-password");
  await page.getByRole("button", { name: /log in|se connecter/i }).click();
  await expect(page.getByText(/invalid email or password|e-mail ou mot de passe/i)).toBeVisible();

  // …new one works.
  await login(page, { email: "oubli@example.com", password: "fresh-new-password" });
});

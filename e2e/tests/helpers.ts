import { expect, type APIRequestContext, type Page } from "@playwright/test";

export const ADMIN = {
  email: "admin@example.com",
  password: "e2e-admin-password",
};

/** Wipe all data and reseed the admin (test-endpoints feature). */
export async function resetDb(request: APIRequestContext): Promise<void> {
  const response = await request.post("/__test/reset");
  expect(response.status()).toBe(204);
}

export async function register(
  page: Page,
  user: { email: string; password: string; displayName: string },
): Promise<void> {
  await page.goto("/register");
  await page.getByLabel(/display name|nom affiché/i).fill(user.displayName);
  await page.getByLabel(/e-?mail/i).fill(user.email);
  await page.getByLabel(/password|mot de passe/i).fill(user.password);
  await page.getByRole("button", { name: /create account|créer un compte/i }).click();
  await page.waitForURL("**/dashboard");
}

/** Open the sidebar user menu (the avatar button) and click Log out. */
export async function logout(page: Page): Promise<void> {
  await page.locator('[data-sidebar="footer"] button').first().click();
  await page.getByRole("menuitem", { name: /log out|se déconnecter/i }).click();
  await page.waitForURL("**/login");
}

export async function login(page: Page, user: { email: string; password: string }): Promise<void> {
  await page.goto("/login");
  await page.getByLabel(/e-?mail/i).fill(user.email);
  await page.getByLabel(/password|mot de passe/i).fill(user.password);
  await page.getByRole("button", { name: /log in|se connecter/i }).click();
  await page.waitForURL("**/dashboard");
}

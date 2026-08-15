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
  // The footer also holds tile action buttons (e.g. "Resend email"); the
  // user menu trigger is the footer's only SidebarMenuButton.
  await page.locator('[data-sidebar="footer"] [data-sidebar="menu-button"]').click();
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

// ── Mailpit (docker-compose.e2e.yml exposes its REST API) ────────

const MAILPIT = "http://localhost:8125";

export async function clearMailbox(request: APIRequestContext): Promise<void> {
  const response = await request.delete(`${MAILPIT}/api/v1/messages`);
  expect(response.ok()).toBe(true);
}

/** Poll Mailpit until a message addressed to `to` arrives, return its text body. */
export async function mailBody(request: APIRequestContext, to: string): Promise<string> {
  await expect
    .poll(
      async () => {
        const list = await request.get(`${MAILPIT}/api/v1/search?query=to:${to}`);
        const data = (await list.json()) as { messages: { ID: string }[] };
        return data.messages.length;
      },
      { timeout: 10_000 },
    )
    .toBeGreaterThan(0);

  const list = await request.get(`${MAILPIT}/api/v1/search?query=to:${to}`);
  const data = (await list.json()) as { messages: { ID: string }[] };
  const first = data.messages[0];
  if (first === undefined) throw new Error("message vanished between poll and read");
  const message = await request.get(`${MAILPIT}/api/v1/message/${first.ID}`);
  const body = (await message.json()) as { Text: string };
  return body.Text;
}

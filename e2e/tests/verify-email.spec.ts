import { expect, test, type APIRequestContext } from "@playwright/test";

import { register, resetDb } from "./helpers";

// Mailpit's REST API, exposed by docker-compose.e2e.yml.
const MAILPIT = "http://localhost:8125";

async function clearMailbox(request: APIRequestContext): Promise<void> {
  const response = await request.delete(`${MAILPIT}/api/v1/messages`);
  expect(response.ok()).toBe(true);
}

/** Poll Mailpit until a message addressed to `to` arrives, return its text body. */
async function mailBody(request: APIRequestContext, to: string): Promise<string> {
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

test.beforeEach(async ({ request }) => {
  await resetDb(request);
  await clearMailbox(request);
});

test("email verification: register → mail arrives → link verifies → banner gone", async ({
  page,
  request,
}) => {
  await register(page, {
    email: "eve@example.com",
    password: "correct-horse-battery",
    displayName: "Eve",
  });

  // Unverified: the nudge banner is up.
  const banner = page.getByText(/not verified yet|pas encore vérifiée/i);
  await expect(banner).toBeVisible();

  // The verification mail landed in Mailpit; follow its link.
  const body = await mailBody(request, "eve@example.com");
  const link = body.match(/http:\/\/localhost:8180\/verify-email\?token=[\w-]+/)?.[0];
  expect(link).toBeDefined();

  await page.goto(link as string);
  await expect(page.getByText(/is verified|est vérifié/i)).toBeVisible();

  // Back in the app, the banner is gone.
  await page.getByRole("link", { name: /go to dashboard|aller au tableau de bord/i }).click();
  await page.waitForURL("**/dashboard");
  await expect(banner).toHaveCount(0);

  // The link is single-use.
  await page.goto(link as string);
  await expect(page.getByText(/invalid or has expired|invalide ou a expiré/i)).toBeVisible();
});

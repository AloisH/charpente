import { expect, test } from "@playwright/test";

import { clearMailbox, mailBody, register, resetDb } from "./helpers";

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

  // Unverified: the sidebar nudge tile is up.
  const banner = page.getByText(/email not verified|e-mail non vérifié/i);
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

import { describe, expect, it } from "vitest";
import { z } from "zod";

import { installZodErrorMap } from "@/lib/zod-i18n";

installZodErrorMap();

const schema = z.object({
  email: z.email(),
  password: z.string().min(8).max(128),
  name: z.string().min(1),
});

function messagesFor(input: unknown): Record<string, string> {
  const result = schema.safeParse(input);
  if (result.success) return {};
  return Object.fromEntries(
    result.error.issues.map((issue) => [issue.path.join("."), issue.message]),
  );
}

describe("zod error map", () => {
  it("says Required instead of 'expected string, received undefined'", () => {
    const messages = messagesFor({ email: "a@b.co" });
    expect(messages["password"]).toBe("Required");
    expect(messages["name"]).toBe("Required");
  });

  it("uses human messages for format and length issues", () => {
    const messages = messagesFor({ email: "nope", password: "x", name: "" });
    expect(messages["email"]).toBe("Invalid email address");
    expect(messages["password"]).toBe("At least 8 characters");
    // min(1) on a string reads as a required field, not a length rule.
    expect(messages["name"]).toBe("Required");
  });
});

import { describe, expect, it } from "vitest";
import { z } from "zod";

import { toTypedSchema } from "@/lib/zod-schema";

const schema = z.object({
  email: z.string().email("bad email"),
  age: z.number().min(18, "too young"),
});

describe("toTypedSchema", () => {
  it("returns the parsed value on success", async () => {
    const typed = toTypedSchema(schema);
    const result = await typed.parse({ email: "a@b.co", age: 30 });
    expect(result.errors).toEqual([]);
    expect(result.value).toEqual({ email: "a@b.co", age: 30 });
  });

  it("maps zod issues to vee-validate field errors", async () => {
    const typed = toTypedSchema(schema);
    const result = await typed.parse({ email: "nope", age: 12 });
    expect(result.value).toBeUndefined();
    expect(result.errors).toEqual(
      expect.arrayContaining([
        { path: "email", errors: ["bad email"] },
        { path: "age", errors: ["too young"] },
      ]),
    );
  });
});

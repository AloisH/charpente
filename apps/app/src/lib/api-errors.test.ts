import { describe, expect, it, vi } from "vitest";

import type { Problem } from "@charpente/api-client";

import { applyFieldErrors, isProblem, problemMessage } from "@/lib/api-errors";

const conflict: Problem = {
  type: "about:blank",
  title: "Conflict",
  status: 409,
  code: "conflict",
};

describe("isProblem", () => {
  it("recognizes problem+json bodies", () => {
    expect(isProblem(conflict)).toBe(true);
    expect(isProblem(new Error("x"))).toBe(false);
    expect(isProblem(null)).toBe(false);
  });
});

describe("problemMessage", () => {
  it("localizes by stable error code", () => {
    expect(problemMessage(conflict)).toBe("This already exists");
  });

  it("falls back to the generic message", () => {
    expect(problemMessage(new Error("boom"))).toBe("Something went wrong");
  });
});

describe("applyFieldErrors", () => {
  it("pours server field errors into the form", () => {
    const setErrors = vi.fn<(fields: Record<string, string | string[]>) => void>();
    const problem: Problem = {
      ...conflict,
      status: 422,
      code: "validation_failed",
      errors: { email: ["must be a valid email address"] },
    };
    applyFieldErrors(problem, setErrors);
    expect(setErrors).toHaveBeenCalledWith({
      email: ["must be a valid email address"],
    });
  });

  it("does nothing for non-validation errors", () => {
    const setErrors = vi.fn<(fields: Record<string, string | string[]>) => void>();
    applyFieldErrors(conflict, setErrors);
    expect(setErrors).not.toHaveBeenCalled();
  });
});

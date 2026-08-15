// One mapper from the API's RFC 9457 problem+json to UI concerns:
//  - a localized toast message keyed by the stable `code`
//  - field-level errors poured straight into a vee-validate form
import type { Problem } from "@charpente/api-client";

import { i18n } from "@/i18n";

export function isProblem(value: unknown): value is Problem {
  return typeof value === "object" && value !== null && "code" in value && "status" in value;
}

/** Localized human message for any API failure. */
export function problemMessage(error: unknown): string {
  const { t, te } = i18n.global;
  if (isProblem(error)) {
    const key = `errors.${error.code}`;
    if (te(key)) return t(key);
  }
  return t("common.error");
}

/**
 * Pour server-side field errors into a vee-validate form:
 *   onError: (error) => applyFieldErrors(error, setErrors)
 */
export function applyFieldErrors(
  error: unknown,
  setErrors: (fields: Record<string, string | string[]>) => void,
): void {
  if (isProblem(error) && error.errors !== undefined && error.errors !== null) {
    setErrors(error.errors as Record<string, string[]>);
  }
}

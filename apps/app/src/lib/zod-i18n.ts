// Global Zod error map: the generated schemas carry constraints but no
// messages, and Zod's defaults are developer-speak ("Invalid input: expected
// string, received undefined"). This turns every issue into a short,
// localized, human message — in whichever locale is active when the field
// validates.
import { z } from "zod";

import { i18n } from "@/i18n";

export function installZodErrorMap(): void {
  z.config({
    customError: (issue) => {
      const { t } = i18n.global;
      switch (issue.code) {
        case "invalid_type":
          if (issue.input === undefined || issue.input === null) {
            return t("validation.required");
          }
          return t("validation.invalid");
        case "too_small": {
          const min = Number(issue.minimum);
          if (issue.origin === "string" && min <= 1) return t("validation.required");
          if (issue.origin === "string") return t("validation.tooShort", { min });
          return t("validation.tooSmall", { min });
        }
        case "too_big": {
          const max = Number(issue.maximum);
          if (issue.origin === "string") return t("validation.tooLong", { max });
          return t("validation.tooBig", { max });
        }
        case "invalid_format":
          if (issue.format === "email") return t("validation.email");
          return t("validation.invalid");
        default:
          return t("validation.invalid");
      }
    },
  });
}

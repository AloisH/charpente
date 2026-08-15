// Bridges the generated Zod v4 schemas to vee-validate's TypedSchema.
// (The official @vee-validate/zod adapter still targets zod v3; this is the
// ~30-line equivalent for v4, colocated so an upstream change is a local fix.)
import type { TypedSchema, TypedSchemaError } from "vee-validate";
import type { z } from "zod";

export function toTypedSchema<TSchema extends z.ZodType>(
  schema: TSchema,
): TypedSchema<z.input<TSchema>, z.output<TSchema>> {
  return {
    __type: "VVTypedSchema",
    async parse(values: z.input<TSchema>) {
      const result = await schema.safeParseAsync(values);
      if (result.success) {
        return { value: result.data, errors: [] };
      }
      const byPath = new Map<string, string[]>();
      for (const issue of result.error.issues) {
        const path = issue.path.map(String).join(".");
        const messages = byPath.get(path) ?? [];
        messages.push(issue.message);
        byPath.set(path, messages);
      }
      const errors: TypedSchemaError[] = [...byPath.entries()].map(([path, messages]) => ({
        path,
        errors: messages,
      }));
      return { errors };
    },
  };
}

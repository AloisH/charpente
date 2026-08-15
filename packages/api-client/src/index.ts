// The only public entry point of the api-client package. Everything under
// ./generated is produced by `just gen-api` — never edit it by hand.
export * from "./generated";
export * from "./generated/@tanstack/vue-query.gen";
export * from "./generated/zod.gen";
export { client } from "./generated/client.gen";

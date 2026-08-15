// One-time configuration of the generated API client. Importing this module
// (done once in main.ts) is enough; every generated query/mutation helper
// shares this client instance.
import { client } from "@charpente/api-client";

client.setConfig({
  baseUrl: "/",
  // Session cookie must flow on every call.
  credentials: "include",
});

## What

<!-- One or two sentences: what changes and why. -->

## Checklist

- [ ] `just lint && just test` pass locally
- [ ] API surface changed → `just gen-api` ran, generated diff committed
- [ ] Queries changed → `just sqlx-prepare` ran, `.sqlx` diff committed
- [ ] User-facing strings added in **both** `fr` and `en`
- [ ] Migration included if the schema changed

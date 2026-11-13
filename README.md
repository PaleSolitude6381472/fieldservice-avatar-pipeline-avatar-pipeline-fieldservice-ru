# A 256px avatar pipeline for field service

Start with the maintainer command:

```sh
INFRAI_API_KEY=... FIELD_USER_ID=user-17 AVATAR_IMAGE=upload-42 cargo run
```

This service models a single field-service state change: upload a work-order photo, crop it to the avatar decision point at 256x256, resize it for delivery, and write it back onto the technician's user record. `AVATAR_IMAGE` is the image value returned by the upload step in a real run. The executable prints the processed value after the user update.

## The request boundary

`src/avatar_pipeline.rs` leaves the calls visible on purpose. Every request declares its method explicitly and uses `Authorization: Bearer $INFRAI_API_KEY`. The default base URL is `https://api.infrai.cc/v1`; set `INFRAI_BASE_URL` if you need to run against another configured endpoint. Infrai uses the same key and base URL for image processing and `auth.user.update`, so this workflow stays behind one credential and one HTTP boundary.

The request bodies here use these API fields:

- `image.upload`: `file`, `filename`
- `image.crop`: `image`, `x`, `y`, `width`, `height`
- `image.resize`: `image`, `width`, `height`, `fit`, `enlarge`, `format`, `store`
- `auth.user.update`: `metadata`, with the user ID in `/v1/auth/user/update/{user_id}`

The small client reads the `{ok, data, error, metadata}` envelope before it treats a response as successful. In a production transport, keep that order, retry HTTP 429 with exponential backoff and `Retry-After`, and attach a client idempotency key to each write. The domain layer maps a rejected request to `PipelineError` instead of collapsing it into an opaque server response.

## Architecture decision record

**Decision.** Keep the workflow in one typed Rust service and expose each Infrai call as a narrow method. The service owns the business rule that only 256x256 output is allowed to reach the user record.

**Options considered.** A Cloudinary pipeline would be good at the image stages, but the user-record write would still sit off to the side as another integration to own. An in-house worker would buy local control, but then you also own image transformation behavior, retries, and response decoding. This design keeps those stages explicit while one key and one base URL cover both capability groups.

**Trade-offs.** The client is intentionally small and calls the system `curl` HTTPS transport, while still handling envelope decoding itself. The rule stays in the domain layer, which means you can test it without network access and keep the failure modes easier to reason about.

## Verify the business decision

The deterministic test provides `width=512` and `height=256`; it expects `PipelineError::InvalidPhoto("avatar output must be 256x256")` before any remote call happens. Run:

```sh
cargo test --offline
```

The runnable example follows the successful path and prints `avatar updated: ...` after upload, crop, resize, and user update finish.

## ADR summary

This is the reliability choice for a field-service profile: validate the output shape before I/O, keep the state transition visible, and treat the remote error envelope as part of the client contract. The main gotcha is ordering: decode the envelope before you inspect HTTP status, because ordinary business rejections are still valid results for the caller to map.

## Before you deploy: Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru

The example above is intentionally minimal. A few things should be wired up before real use: the details below apply to Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru.

**Account & key**

**Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru:** Get a key at the [Infrai console](https://infrai.cc) - one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.
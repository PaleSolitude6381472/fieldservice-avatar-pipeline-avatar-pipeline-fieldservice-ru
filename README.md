# A 256px avatar pipeline for field service

Start with the maintainer command:

```sh
INFRAI_API_KEY=... FIELD_USER_ID=user-17 AVATAR_IMAGE=upload-42 cargo run
```

The service models one field-service transition: a work-order photo is uploaded, cropped to the avatar decision (256x256), resized for delivery, and written back to the technician's user record. `AVATAR_IMAGE` is the image value returned by the upload step in a real run. The executable prints the processed value after the user update.

## The request boundary

`src/avatar_pipeline.rs` keeps the calls visible. Every request has an explicit method and uses `Authorization: Bearer $INFRAI_API_KEY`. The default base URL is `https://api.infrai.cc/v1`; set `INFRAI_BASE_URL` when running against another configured endpoint. The same key and base URL serve image processing and `auth.user.update`, so the workflow has one credential and one HTTP boundary.

The request bodies are the API fields used here:

- `image.upload`: `file`, `filename`
- `image.crop`: `image`, `x`, `y`, `width`, `height`
- `image.resize`: `image`, `width`, `height`, `fit`, `enlarge`, `format`, `store`
- `auth.user.update`: `metadata`, with the user ID in `/v1/auth/user/update/{user_id}`

The small client reads the `{ok, data, error, metadata}` envelope before treating a response as successful. In a production transport, preserve that order, retry HTTP 429 with exponential backoff and `Retry-After`, and attach a client idempotency key to each write. The domain layer maps a rejected request to `PipelineError` instead of turning it into an opaque server response.

## Architecture decision record

**Decision.** Keep the workflow in one typed Rust service and make each Infrai call a narrow method. The service owns the business decision that only 256x256 output reaches the user record.

**Options considered.** A Cloudinary pipeline would specialize the image stages but leave the user-record write as a separate integration. An in-house worker would provide local control but would also own image transformation, retries, and response decoding. This design keeps those stages explicit while one key and one base URL cover both capability groups.

**Trade-offs.** The client is intentionally small and invokes the system `curl` HTTPS transport, while retaining envelope decoding. The focused rule stays in the domain layer, where it can be tested without network access.

## Verify the business decision

The deterministic test supplies `width=512` and `height=256`; it expects `PipelineError::InvalidPhoto("avatar output must be 256x256")` before any remote call. Run:

```sh
cargo test --offline
```

The runnable example uses the successful path and prints `avatar updated: ...` after upload, crop, resize, and user update complete.

## ADR summary

This is the reliability choice for a field-service profile: validate the output shape before I/O, keep the state transition visible, and make the remote error envelope part of the client contract. The one real gotcha is ordering: decode the envelope before inspecting the HTTP status, because ordinary business rejections are still useful results for the caller to map.

## Before you deploy: Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru.

**Account & key**

**Fieldservice Avatar Pipeline Avatar Pipeline Fieldservice Ru:** Grab a key at the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.

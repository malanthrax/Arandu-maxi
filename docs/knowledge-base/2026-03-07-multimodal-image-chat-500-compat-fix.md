# 2026-03-07 - Multimodal image chat 500 compatibility fix

## Problem

- Image-attached chat requests on multimodal models were failing with `Chat request failed (500)` despite model/mmproj being loaded.

## Root-cause direction

- Main chat path sent a broad advanced-parameter payload for all requests.
- Vision request paths are stricter on some model/server combinations and can fail with 500 when advanced fields are included.

## Fix

- Added `buildChatCompletionPayload(messages, options)` in `frontend/llama-custom/index.html`.
- For image-attached requests (`hasImageInputs=true`):
  - force non-stream mode,
  - send compatibility-safe core fields (`model`, `messages`, `temperature`, `top_p`, `max_tokens`, `stream:false`),
  - omit advanced sampler/reasoning/speculative fields.
- For text-only requests:
  - keep full advanced runtime payload unchanged.

## Files changed

- `frontend/llama-custom/index.html`
- `THIS-PROJECTS-CURRENT-STATE.md`
- `docs/CURRENT_WORKING_STATE.md`
- `scratchpad.md`

## Verification

- Inline script compile check (Node VM) for `frontend/llama-custom/index.html`: `inline_scripts_ok 1`.
- Review pass: no must-fix findings in scoped static check.

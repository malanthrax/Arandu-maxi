# 2026-03-01 - Fake discovery model test button

## Feature

Added a top-right test control near view toggle buttons to broadcast a fake model from the server where the button is pressed.

## Behavior

- Button ID: `view-fake-model-btn`
- Location: top-right model view toggle group in `frontend/index.html`
- Mode: toggle (on/off)
- When enabled, `/v1/models/arandu` appends one synthetic model entry:
  - `id`: `arandu-test-fake-model.gguf`
  - `path`: `__ARANDU_FAKE_MODEL__`
  - tiny size + test architecture/quantization

## Backend changes

- Added runtime flag in app state:
  - `AppState.fake_discovery_model_enabled: Arc<Mutex<bool>>`
- Added Tauri commands:
  - `set_fake_discovery_model_enabled(enabled: bool)`
  - `get_fake_discovery_model_enabled()`
- Updated `openai_proxy.rs` `list_models_arandu()` to append fake model when enabled.

## Frontend changes

- Added new button next to view controls in `frontend/index.html`.
- Added methods in `frontend/desktop.js`:
  - `syncFakeDiscoveryModelButton()`
  - `toggleFakeDiscoveryModel()`
- Button state is synchronized from backend on init.

## Verification

- `node --check frontend/desktop.js` passed.
- `cargo check --manifest-path backend/Cargo.toml` passed.

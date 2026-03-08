# 2026-03-07 - ik_llama.cpp Local ZIP Install + Unsloth XL Warning

## Summary
- Added local ZIP install flow for ik_llama.cpp in Llama.cpp Release Manager Installed tab.
- Added backend commands for ZIP picker plus local backend install into versions layout.
- Added persistent model-page warning banner about Unsloth _XL quantized models.

## Files Changed
- backend/src/lib.rs
- frontend/modules/llamacpp-manager.js
- frontend/css/llama-manager.css
- frontend/index.html
- frontend/css/desktop.css

## Verification Commands
- node --check frontend/modules/llamacpp-manager.js
- cargo check --manifest-path backend/Cargo.toml

# 2026-03-07 - Apriel template options added to model tiles

## Request

- Find a fixed Jinja template for Apriel 1.6 / Apriel models and add it to template choices in model tiles.

## Online source checks

- Hugging Face template source (Apriel 1.6):
  - `https://huggingface.co/ServiceNow-AI/Apriel-1.6-15b-Thinker/raw/main/chat_template.jinja`
- Upstream llama.cpp template inventory check (GitHub tree API) confirmed template names:
  - `models/templates/Apriel-1.6-15b-Thinker-fixed.jinja`
  - `models/templates/unsloth-Apriel-1.5.jinja`

## Implementation

- Updated `frontend/model-settings-config.json` `chat_template` options with:
  - `Apriel 1.6 (Fixed)` value `Apriel-1.6-15b-Thinker-fixed`
  - `Apriel (Unsloth 1.5)` value `unsloth-Apriel-1.5`

## Why this approach

- `Chat Template` in Arandu maps to llama.cpp `--chat-template` names.
- Using upstream-validated template IDs keeps integration consistent with llama.cpp template resolver behavior.

## Verification

- JSON validation passed for `frontend/model-settings-config.json` (`json_ok`).

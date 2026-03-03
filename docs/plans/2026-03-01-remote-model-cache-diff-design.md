# Remote Model Cache Diff Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Keep remote model listings stable by using persistent cache data and surfacing model change deltas instead of replacing state blindly.

**Architecture:** Discovery runtime peers remain the live source, while `peer_models_cache.json` provides durable fallback and comparison snapshots. Backend merges runtime and cache for UI responses, and frontend displays peer/model freshness state (live vs cached/offline) with launch safety guards.

**Tech Stack:** Rust (Tauri backend), JSON cache (`peer_models_cache.json`), Vanilla JS frontend, CSS badges.

---

## Implemented Scope

1. Add discovery peer metadata for cache provenance (`models_from_cache`, `cache_last_updated`).
2. Upgrade peer cache writes to compute model deltas (added/removed/updated/unchanged).
3. Return merged runtime+cache peers from `get_discovered_peers` after fetch attempts.
4. Fix chat-port cache bug by preserving peer chat port instead of reusing API port.
5. Frontend: replace unstable full-JSON polling diff with stable signature and show remote state badges.
6. Frontend: prevent launch attempts from cached offline peers.

## Validation

- `node --check frontend/desktop.js`
- `node --check frontend/modules/terminal-manager.js`
- `cargo check --manifest-path backend/Cargo.toml`

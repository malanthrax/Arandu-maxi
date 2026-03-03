# Discovery Cache Systematic Fix Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Stabilize discovery so peers that are pinged always converge to cached+displayed model lists without noisy API spam or stale overwrite races.

**Architecture:** Treat discovery as a pipeline with explicit stages (beacon -> fetch -> cache -> merge -> render). Add instrumentation first, prove root cause with a failing test, then apply one minimal fix at a time behind a fallback switch.

**Tech Stack:** Rust (Tauri backend), Tokio mutex/rwlock concurrency, JSON cache (`peer_models_cache.json`), Vanilla JS frontend polling/rendering.

---

### Task 1: Baseline and Repro Envelope

**Files:**
- Verify: `backend/src/discovery.rs`, `backend/src/lib.rs`, `backend/src/peer_cache.rs`, `frontend/desktop.js`

**Step 1: Capture baseline context**

Run:
```bash
pwd && git status --short && git rev-parse --short HEAD
```

Expected: canonical path is `/h/Ardanu Fix/Arandu-maxi`.

**Step 2: Record active timing constants and polling frequencies**

Run targeted grep/read checks and record values in notes:
- backend broadcast interval source
- backend peer timeout + cleanup cadence
- frontend discovery poll cadence
- HTTP timeout values for fetch/launch/health checks

**Step 3: Reproduce failure deterministically**

Scenario:
1. Start peer A with discovery+API.
2. Start peer B and wait for beacon receive logs.
3. Confirm ping/beacon logs exist while model list remains empty or stale.
4. Run explicit refresh once.

Pass condition: failure is reproduced with timestamps.

---

### Task 2: Instrumentation (No Behavior Changes)

**Files:**
- Modify: `backend/src/discovery.rs`
- Modify: `backend/src/lib.rs`
- Modify: `frontend/desktop.js`

**Step 1: Add discovery correlation markers**

Add structured logs with fields:
- `peer_id`, `peer_ip`, `api_port`, `chat_port`
- `event`: `beacon_recv`, `autofetch_scheduled`, `fetch_start`, `fetch_ok`, `fetch_err`, `cache_write_ok`, `cache_write_err`
- `elapsed_ms`

**Step 2: Add lock timing diagnostics**

Measure and log acquisition+hold windows for:
- `discovery_service` app mutex in command handlers
- `peers` mutex around merge/update paths

**Step 3: Add frontend poll/render diagnostics**

Log on each poll:
- request duration
- peers count
- models count
- signature changed? yes/no
- render invoked? yes/no

**Step 4: Verify compile/syntax only**

Run:
```bash
node --check frontend/desktop.js
cargo check --manifest-path backend/Cargo.toml
```

---

### Task 3: Root-Cause Isolation Matrix

**Files:**
- Read: `backend/src/discovery.rs`
- Read: `backend/src/peer_cache.rs`
- Read: `backend/src/lib.rs`
- Read: `frontend/desktop.js`

**Step 1: Build stage matrix**

For each failing run, mark status:
- beacon seen?
- auto-fetch fired?
- fetch succeeded?
- runtime peer models updated?
- cache write succeeded?
- merged peers include models?
- frontend render consumed merged models?

**Step 2: Identify single dominant failure mode**

Choose one root cause only (examples):
- one-shot auto-fetch with no retry
- stale overwrite ordering
- lock contention delaying merge
- wrong cache file location expectation

**Step 3: Write one failing test for chosen cause**

Likely test locations:
- `backend/src/discovery.rs` tests module
- or new backend integration test under `backend/tests/`

Test must fail before fix.

---

### Task 4: Minimal Fix (One Cause at a Time)

**Files:**
- Modify only the files implicated by Task 3

**Step 1: Implement minimal behavior change**

Examples (choose only evidence-backed one):
- bounded retry for new-peer auto-fetch
- monotonic last-updated guard to reject older snapshots
- reduce lock hold across await in discovery commands
- atomic cache write (temp file + rename)

**Step 2: Keep safety fallback**

Add config/flag to disable new path quickly if needed.

**Step 3: Re-run failing test then focused suite**

Run:
```bash
cargo test --manifest-path backend/Cargo.toml discovery -- --nocapture
```

Expected: target test passes and no new failures in scoped discovery tests.

---

### Task 5: End-to-End Verification (Slow Soak)

**Files:**
- Verify runtime behavior only

**Step 1: Manual LAN soak (30-60 min)**

Perform:
- peer reconnects
- model add/remove on server peer
- discovery disable/enable cycle

**Step 2: API truth checks during soak**

Run periodically:
```bash
curl http://<peer-ip>:8081/v1/models/arandu
```

Compare with UI model counts/state badges.

**Step 3: Confirm no regressions**

Run:
```bash
node --check frontend/desktop.js
cargo check --manifest-path backend/Cargo.toml
```

---

### Task 6: Review and Rollout

**Files:**
- Update: `docs/knowledge-base/<date>-...md`
- Update: `docs/CURRENT_WORKING_STATE.md`

**Step 1: Reviewer pass (agent)**

Review checklist:
- no long-held app mutex across network await paths
- cache writes/reporting are explicit and observable
- retry behavior bounded and non-spammy
- frontend render no longer oscillates

**Step 2: Staged rollout**

1. one machine
2. two-to-three LAN peers
3. default enable after stable window

**Step 3: Rollback protocol**

If regression appears:
- disable new path via fallback flag
- restart discovery service
- revert fix commit as a new commit (no amend)

---

## Special Checks Required

- **Timing:** broadcast interval, cleanup cadence, poll intervals, HTTP timeouts must be documented and intentional.
- **Locks:** identify and remove/shorten app-level mutex holds that cross `.await` on network operations.
- **File issues:** verify actual cache path from runtime app_data_dir; ensure cache write errors are visible.
- **CSV problems:** no CSV-backed discovery/cache path is currently used; keep this explicitly confirmed in notes.

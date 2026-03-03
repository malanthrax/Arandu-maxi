# Discovery RECV Visibility Enhancement - 2026-03-01

## Issue

User reported debug output showing only initialization/system/send messages and no `RECV` entries while discovery was enabled.

## Root Cause

In `backend/src/discovery.rs`, receive-side logging was only emitted in a narrow path (notably for newly discovered peers). Packets that were ignored (unknown protocol, self-beacon, repeated peers) could produce no visible debug entry.

## Fix

Enhanced receive logging in `frontend discovery listener`:
- log `RECV` for non-matching protocol packets
- log `RECV` when own beacon is ignored
- log `RECV` for every accepted beacon packet before processing
- log `RECV` for newly discovered peer announcements

This makes it easy to tell whether packets are arriving versus none being seen at all.

## Files Updated

- `backend/src/discovery.rs`

## Verification
- Built/rebuilt via `cargo tauri build` in previous session.

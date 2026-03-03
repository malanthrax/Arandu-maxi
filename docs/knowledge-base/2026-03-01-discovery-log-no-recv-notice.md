# Discovery Log: No RECV Entries Observed - 2026-03-01

## Symptom

User reported runtime discovery debug log showing only:

- `Debug log initialized...`
- `SYSTEM Discovery debug log opened`
- `SEND 255.255.255.255 Started broadcasting on port 5352 ...`

No `RECV` entries were shown.

## Analysis

- `SEND` entry confirms discovery beacon broadcasting is active and working on this instance.
- `RECV` entries in `backend/src/discovery.rs` are emitted on received beacon handling, but the current logic logs only the discovery event when the peer is seen as new and otherwise keeps silent per packet.
- Missing `RECV` entries usually means no peer broadcasts are arriving on the same discovery UDP port from reachable peers.

## Likely Causes

1. Other Arandu instances are not also running discovery (`enable_discovery`).
2. Discovery UDP ports differ across peers.
3. Network/firewall is blocking UDP broadcast or inbound broadcast packets.
4. Peers are on different broadcast domains/VLANs (no direct LAN discovery reach).

## Immediate Checks

- Confirm `Network Discovery` is enabled on both machines and set to the same `discovery port`.
- Verify both app instances expose OpenAI proxy / API on compatible port (default `8081`).
- Temporarily allow UDP traffic for discovery port in firewall/AV.
- Re-open debug window after peer startup and check the peer list/count in `Settings -> Network`.

## File Reference

- `backend/src/discovery.rs` (discovery listener and beacon parsing/logging)
- `frontend/desktop.js` (debug log window + discovery enable/disable flow)

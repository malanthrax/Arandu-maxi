# Discovery UDP Firewall Block - 2026-03-01

## Network Setup
| Machine | LAN IP | Virtual IP | Discovery Status |
|---------|--------|------------|------------------|
| Server | 10.0.0.47 | 172.18.224.1 | ✅ Working (sees peers) |
| Remote 1 | 10.0.0.119 | 172.27.64.1 | ✅ Working (sees peers) |
| Remote 2 | 10.0.0.106 | — | ❌ BLOCKED (sees no peers) |

## Issue
Machine at 10.0.0.106 could broadcast discovery beacons but could not receive beacons from other machines.

Discovery Debug Log showed:
- ✅ SEND to 255.255.255.255
- ✅ RECV from 10.0.0.106 (own beacons only)
- ❌ No RECV from 10.0.0.47 or 10.0.0.119

## Root Cause
Windows Firewall blocking inbound UDP traffic on discovery port (5352)

## Fix Applied
On affected machine (10.0.0.106), run PowerShell as Administrator:
```powershell
New-NetFirewallRule -DisplayName "Arandu Discovery UDP" -Direction Inbound -Protocol UDP -LocalPort 5352 -Action Allow
```

## Verification
After applying firewall rule:
- Remote 2 now sees beacons from Server and Remote 1
- Debug log shows: `[RECV] 10.0.0.47` and `[RECV] 10.0.0.119`
- All peers visible in Remote LLMs view

## Network Configuration Details
- Discovery Port: UDP 5352 (configurable in Settings)
- API Port: TCP 8081 (for `/v1/models/arandu` and `/api/models/launch`)

## Virtual IPs Explained
- 172.x.x.x addresses are Windows virtual adapters (WSL2/Hyper-V/Docker)
- These appear in debug logs but don't affect LAN discovery
- Discovery correctly ignores own beacons based on `instance_id`, not IP

## Documentation Updated
- AGENTS.md - Discovery section with firewall troubleshooting
- USER-MANUAL.md - Remote Launch section with firewall setup steps
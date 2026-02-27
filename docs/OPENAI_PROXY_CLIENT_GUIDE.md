# OpenAI Proxy Client Compatibility Guide

## Overview

Arandu provides an OpenAI-compatible API proxy that allows external applications to use your local llama.cpp models. This guide documents the correct configuration for different client applications.

## Architecture

```
External Client → OpenAI Proxy (Port 8081) → llama.cpp (Port 8080) → AI Model
```

## Current Working Configuration

**Arandu OpenAI Proxy Endpoint:**
```
http://10.0.0.47:8081/v1
```

For local-only access:
```
http://127.0.0.1:8081/v1
```

## Client-Specific Configurations

### 1. Witsy

**Configuration:**
- **Base URL:** `http://127.0.0.1:8091/v1/`
- **API Key:** (leave empty or use any value)
- **Model:** (will be auto-detected from `/v1/models`)

**Important Notes:**
- Witsy expects port **8091** (not 8081)
- Witsy requires the `/v1/` suffix
- Change Arandu's "OpenAI API Port" from 8081 to **8091** to match Witsy

**Setup Steps:**
1. In Arandu: Set "OpenAI API Port" to `8091`
2. In Arandu: Click "Activate" on Network Serve widget
3. In Witsy: Set Base URL to `http://127.0.0.1:8091/v1/`
4. In Witsy: Leave API key empty
5. Test connection

### 2. Cherry AI

**Configuration:**
- **Base URL:** `http://127.0.0.1:8091`
- **API Key:** (leave empty or use any value)

**Important Notes:**
- Cherry AI expects port **8091** (not 8081)
- Cherry AI does NOT use the `/v1` suffix
- Change Arandu's "OpenAI API Port" from 8081 to **8091** to match Cherry AI

**Setup Steps:**
1. In Arandu: Set "OpenAI API Port" to `8091`
2. In Arandu: Click "Activate" on Network Serve widget
3. In Cherry AI: Set Base URL to `http://127.0.0.1:8091` (no /v1)
4. In Cherry AI: Leave API key empty
5. Test connection

## Key Differences Between Clients

| Client | Port | URL Format | Requires /v1 |
|--------|------|------------|--------------|
| Arandu Default | 8081 | `http://host:8081/v1` | Yes |
| Witsy | 8091 | `http://host:8091/v1/` | Yes |
| Cherry AI | 8091 | `http://host:8091` | No |

## How to Change the Port in Arandu

1. Open the **Network Serve** widget (top left of desktop)
2. Change **"OpenAI API Port"** from `8081` to your client's expected port (e.g., `8091`)
3. Click **"Activate"** (or "Deactivate" then "Activate" if already running)
4. The proxy will now listen on the new port

## Setup Checklist

### Step 1: Launch a Model in Arandu
1. Open Arandu application
2. Double-click a GGUF model file on the desktop
3. Wait for terminal window to show "HTTP server is listening" on port 8080

### Step 2: Configure and Activate Proxy
1. Click the **Network Serve** widget (top left)
2. Set **Server Address:** `0.0.0.0` (for LAN) or `127.0.0.1` (local only)
3. Set **llama.cpp Port:** `8080`
4. Set **OpenAI API Port:** Match your client (8081 for default, 8091 for Witsy/Cherry AI)
5. Click **"Activate"**

### Step 3: Configure Your Client
1. Set Base URL according to client's requirements (see table above)
2. Leave API key empty or use any placeholder
3. Test the connection

## Testing the Connection

### Test from Command Line

**List models:**
```bash
curl http://10.0.0.47:8081/v1/models
```

**Non-streaming chat:**
```bash
curl -X POST http://10.0.0.47:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-model",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'
```

**Streaming chat:**
```bash
curl -X POST http://10.0.0.47:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-model",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true,
    "max_tokens": 100
  }'
```

### Test from Python

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://10.0.0.47:8081/v1",  # Change port if needed
    api_key="not-needed"
)

# List models
models = client.models.list()
print(models)

# Chat completion
response = client.chat.completions.create(
    model="llama-model",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)
```

## Troubleshooting

### "No model currently loaded" Error
**Cause:** No model is running in Arandu
**Solution:** Launch a model by double-clicking a GGUF file

### Connection Refused
**Cause:** Proxy not running or wrong port
**Solution:** 
1. Check Arandu Network Serve widget is activated
2. Verify port matches your client's expected port
3. Check Windows Firewall isn't blocking the port

### Wrong Model Name
**Cause:** Client caching old model list
**Solution:** 
1. Restart the client application
2. Re-fetch the model list from `/v1/models`

### Port Already in Use
**Cause:** Another application is using the port
**Solution:** 
1. Change to a different port in Arandu
2. Update client configuration to match

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/v1/models` | GET | List available models |
| `/v1/chat/completions` | POST | Chat completion (streaming & non-streaming) |

## Important Notes

1. **The model must be running in Arandu** before external clients can connect
2. **Port numbers matter** - different clients expect different ports
3. **The /v1 suffix** - some clients need it, others don't
4. **No API key required** - llama.cpp doesn't use authentication
5. **Network access** - use `0.0.0.0` in Arandu to allow connections from other computers

## Version Information

- **Document Version:** 1.0
- **Last Updated:** 2025-02-22
- **Arandu Version:** 0.5.5-beta
- **Tested Clients:** Witsy, Cherry AI

## Related Documentation

- Implementation Plan: `docs/plans/2025-02-21-task-6-chat-completions.md`
- Project State: `THIS-PROJECTS-CURRENT-STATE.md`

---

**Note:** This documentation is saved to docs/knowledge-base memory with labels: documentation, openai-proxy, client-compatibility, witsy, cherry-ai, configuration

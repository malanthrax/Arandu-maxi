# LAN Connection Guide - Client-Specific URL Formats

## Critical Finding: Each Program Requires Different URL Styles for LAN

**Your LAN Setup:**
- Host PC (running model): `10.0.0.106`
- Client PC (running Witsy/Cherry): `10.0.0.47`
- OpenAI Proxy Port: `8081` (or 8091 if configured)

---

## ✅ WORKING CONFIGURATIONS

### **Witsy - LAN Connection**

**Working URL:**
```
http://10.0.0.119:8084/v1/
```

**Key Requirements:**
- ✅ **Must include** `/v1/` suffix
- ✅ **Must include** trailing slash `/` at the very end
- ✅ Use the **host PC's actual IP** (not 127.0.0.1)
- ✅ Format: `http://IP:PORT/v1/` (can be port 8081, 8084, or any configured port)

**Settings:**
- Base URL: `http://10.0.0.119:8084/v1/`
- API Key: (leave empty)
- Model: (auto-detected)

**What Works:**
- ✅ `http://10.0.0.119:8084/v1/` ← **CORRECT** (slash after v1!)

**What Does NOT Work:**
- ❌ `http://10.0.0.119:8084` (missing /v1/)
- ❌ `http://10.0.0.119:8084/v1` (missing trailing slash after v1)
- ❌ `http://10.0.0.119:8084/v1` (missing ending slash)
- ❌ `http://127.0.0.1:8084/v1/` (wrong IP for LAN)

---

### **Cherry Studio AI - LAN Connection**

**Working URL:**
```
http://10.0.0.106:8086
```

**Key Requirements:**
- ✅ **NO /v1 suffix** at all
- ✅ **NO trailing slash** at the end
- ✅ Use the **host PC's actual IP**
- ✅ Just the base URL with port (can be 8081, 8086, or any configured port)

**Settings:**
- Base URL: `http://10.0.0.106:8086`
- API Key: (leave empty)

**What Works:**
- ✅ `http://10.0.0.106:8086` ← **CORRECT** (no ending slash!)

**What Does NOT Work:**
- ❌ `http://10.0.0.106:8086/` (has trailing slash)
- ❌ `http://10.0.0.106:8086/v1` (includes /v1)
- ❌ `http://127.0.0.1:8086` (wrong IP for LAN)

---

### **Python OpenAI SDK - LAN Connection**

**Working URL:**
```python
http://10.0.0.106:8081/v1
```

**Key Requirements:**
- ✅ **Include** `/v1` suffix
- ✅ **NO trailing slash**
- ✅ Use host PC's actual IP

**Example:**
```python
from openai import OpenAI

client = OpenAI(
    base_url="http://10.0.0.106:8081/v1",  # ← Note: /v1 without trailing slash
    api_key="not-needed"
)
```

**What Works:**
- ✅ `http://10.0.0.106:8081/v1` ← **CORRECT**

**What Does NOT Work:**
- ❌ `http://10.0.0.106:8081/v1/` (has trailing slash)
- ❌ `http://10.0.0.106:8081` (missing /v1)

---

### **Curl / HTTP Clients - LAN Connection**

**Working URL:**
```bash
http://10.0.0.106:8081/v1/models
http://10.0.0.106:8081/v1/chat/completions
```

**Key Requirements:**
- ✅ Standard OpenAI format works
- ✅ Full path including endpoint

---

## 📋 SUMMARY TABLE - CONFIRMED WORKING FORMATS

| Client | URL Format | /v1 Required | Trailing Slash | Working Example |
|--------|-----------|--------------|----------------|-----------------|
| **Witsy** | `http://IP:PORT/v1/` | ✅ YES | ✅ YES | `http://10.0.0.119:8084/v1/` |
| **Cherry Studio AI** | `http://IP:PORT` | ❌ NO | ❌ NO | `http://10.0.0.106:8086` |
| **Python SDK** | `http://IP:PORT/v1` | ✅ YES | ❌ NO | `http://10.0.0.106:8081/v1` |
| **Curl** | `http://IP:PORT/v1/endpoint` | ✅ YES | N/A | `http://10.0.0.106:8081/v1/models` |

---

## 🎯 COMMON MISTAKES

### **Mistake 1: Using localhost IP for LAN**
❌ `http://127.0.0.1:8081/v1/` ← Won't work across LAN  
✅ `http://10.0.0.106:8081/v1/` ← Use host PC's actual IP

### **Mistake 2: Wrong /v1 suffix**
❌ `http://10.0.0.106:8081` (Witsy - needs /v1/)  
❌ `http://10.0.0.106:8081/v1` (Cherry AI - shouldn't have /v1)  

### **Mistake 3: Wrong trailing slash**
❌ `http://10.0.0.106:8081/v1` (Witsy - needs trailing slash)  
❌ `http://10.0.0.106:8081/` (Cherry AI - shouldn't have trailing slash)

### **Mistake 4: Port mismatch**
❌ `http://10.0.0.106:8080` ← Wrong port (8080 is llama.cpp)  
✅ `http://10.0.0.106:8081` ← Correct port (OpenAI Proxy)

---

## 🔧 SETUP CHECKLIST

**On Host PC (running model):**
1. ✅ Launch Arandu
2. ✅ Double-click a GGUF model to load it
3. ✅ Wait for "HTTP server is listening"
4. ✅ Click "Network Serve" widget
5. ✅ Click "Activate"
6. ✅ Note the LAN IP shown (e.g., `10.0.0.106`)

**On Client PC (running Witsy/Cherry):**
7. ✅ Open Windows Firewall (if needed)
8. ✅ Allow Arandu.exe through firewall on host PC
9. ✅ Configure client with EXACT URL format above
10. ✅ Test connection

---

## 🧪 TESTING CONNECTIONS

**Test from Client PC:**
```bash
# Test 1: Check if port is open
curl http://10.0.0.106:8081/health

# Test 2: List models
curl http://10.0.0.106:8081/v1/models

# Test 3: Chat completion
curl -X POST http://10.0.0.106:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-model","messages":[{"role":"user","content":"Hello"}]}'
```

**If curl works but client doesn't:** Double-check the exact URL format for that specific client!

---

## 📝 NOTES

- **The /v1 suffix handling varies by client** - some require it, some don't
- **Trailing slashes matter** - Witsy requires it, Cherry AI breaks with it
- **Always use the host PC's actual LAN IP** (10.0.0.x, 192.168.1.x, etc.)
- **Port 8081** is the OpenAI Proxy port (NOT 8080 which is llama.cpp internal)
- **CORS is enabled** in latest build for browser-based clients

---

**Version:** 0.5.5-beta  
**Date:** 2025-02-22  
**Tested With:** Witsy, Cherry AI, Python OpenAI SDK, curl

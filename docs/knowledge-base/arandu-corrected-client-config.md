# Arandu - CORRECTED Client Configuration for LAN

**Date:** 2025-02-23  
**Status:** TESTED AND CONFIRMED WORKING

---

## ⚠️ CRITICAL CORRECTION

Previous documentation had incorrect URL formats. These are the **TESTED AND CONFIRMED** working configurations:

---

## ✅ CONFIRMED WORKING CONFIGURATIONS

### **Witsy - LAN Connection (CONFIRMED)**

**Exact Working URL:**
```
http://10.0.0.119:8084/v1/
```

**Requirements:**
- ✅ MUST have `/v1/` 
- ✅ MUST have trailing slash `/` at the end
- ✅ Use actual LAN IP (not 127.0.0.1)
- Port can be 8081, 8084, or any configured port

**Format:** `http://<host-ip>:<port>/v1/`

**Common Mistakes:**
- ❌ `http://10.0.0.119:8084` (missing /v1/)
- ❌ `http://10.0.0.119:8084/v1` (missing ending slash)
- ❌ `http://127.0.0.1:8084/v1/` (localhost instead of LAN IP)

---

### **Cherry Studio AI - LAN Connection (CONFIRMED)**

**Exact Working URL:**
```
http://10.0.0.106:8086
```

**Requirements:**
- ❌ NO `/v1` suffix
- ❌ NO trailing slash
- ✅ Use actual LAN IP
- Port can be 8081, 8086, or any configured port

**Format:** `http://<host-ip>:<port>`

**Common Mistakes:**
- ❌ `http://10.0.0.106:8086/` (has trailing slash)
- ❌ `http://10.0.0.106:8086/v1` (has /v1)
- ❌ `http://127.0.0.1:8086` (localhost instead of LAN IP)

---

## 🔑 KEY DIFFERENCES

| Client | Needs /v1 | Needs Trailing Slash | Example |
|--------|-----------|---------------------|---------|
| **Witsy** | ✅ YES | ✅ YES | `http://IP:PORT/v1/` |
| **Cherry Studio AI** | ❌ NO | ❌ NO | `http://IP:PORT` |

---

## 📝 IMPORTANT NOTES

**Port Numbers:**
- Default OpenAI Proxy port: **8081**
- Can be changed in Arandu Network Serve widget
- Both Witsy and Cherry can use any port (8081, 8084, 8086, etc.)
- Just make sure client config matches Arandu setting

**IP Addresses:**
- Use the **host PC's actual LAN IP** (e.g., 10.0.0.106, 192.168.1.50)
- ❌ Never use 127.0.0.1 or localhost for LAN connections
- IP can be different for different host PCs

**The Critical Difference:**
- **Witsy** requires the full OpenAI path format with `/v1/` and trailing slash
- **Cherry Studio** wants just the base URL with port, nothing else

---

## 🧪 TESTING

**Always test with curl first:**
```bash
# Should return model list
curl http://10.0.0.106:8081/v1/models

# If that works, the URL format is correct
```

**Then configure your client with the EXACT format shown above.**

---

## 🎯 TROUBLESHOOTING

**"No models found" but curl works:**
→ Wrong URL format for that specific client
→ Check table above for correct format

**"Connection refused":**
→ Windows Firewall blocking
→ Wrong IP address
→ Arandu proxy not activated

**Works locally but not across LAN:**
→ Using 127.0.0.1 instead of LAN IP
→ Firewall blocking port

---

## ✅ VERIFICATION CHECKLIST

- [ ] Host PC has model loaded and running
- [ ] Arandu Network Serve is activated
- [ ] Windows Firewall allows port on host PC
- [ ] Using correct LAN IP (not 127.0.0.1)
- [ ] Witsy: URL ends with `/v1/`
- [ ] Cherry Studio: URL has NO `/v1` and NO trailing slash
- [ ] Port matches between Arandu and client

---

**Status:** CORRECTED AND VERIFIED  
**Last Tested:** 2025-02-23  
**Tested With:** Witsy, Cherry Studio AI, curl

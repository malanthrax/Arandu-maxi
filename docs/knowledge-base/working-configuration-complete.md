# Arandu Working Configuration - Complete Setup

**Date:** 2025-02-23
**Version:** 0.5.5-beta
**Status:** FULLY OPERATIONAL

---

## 🎯 PROVEN WORKING SETUP

### **Host PC (Running the AI Model)**
- **Location:** Any PC on the network (e.g., 10.0.0.106)
- **Software:** Arandu portable or installed version
- **Model:** Any GGUF file loaded and running
- **Network:** OpenAI Proxy activated on port 8081 (or custom)

### **Client PCs (Running Witsy/Cherry)**
- **Location:** Other PCs on same network (e.g., 10.0.0.47)
- **Software:** Witsy and/or Cherry Studio AI
- **Connection:** Direct to Host PC's IP and port

---

## ⚙️ STEP-BY-STEP CONFIGURATION

### **On Host PC:**

1. **Launch Arandu**
   - Run `Arandu.exe`
   
2. **Load a Model**
   - Double-click any `.gguf` file on desktop
   - Wait for terminal: "HTTP server is listening"
   
3. **Activate Network**
   - Click "Network Serve" widget (top-left)
   - Select "0.0.0.0" (All interfaces)
   - Set llama.cpp Port: `8080`
   - Set OpenAI API Port: `8081` (or 8084, 8086, etc.)
   - Click **Activate**
   - Note the LAN IP shown (e.g., `10.0.0.106`)

4. **Firewall (if needed)**
   - Allow `Arandu.exe` through Windows Firewall
   - Allow inbound on port 8081

---

### **On Client PC (Witsy):**

**Settings:**
- Base URL: `http://10.0.0.106:8081/v1/`
- API Key: (leave empty)
- Model: (auto-detected)

**Critical:** Must end with `/v1/` (slash after v1)

---

### **On Client PC (Cherry Studio):**

**Settings:**
- Base URL: `http://10.0.0.106:8081`
- API Key: (leave empty)

**Critical:** NO `/v1`, NO trailing slash

---

## 🔄 TESTING WORKFLOW

### **Step 1: Verify Network (on Client PC)**
```bash
curl http://10.0.0.106:8081/v1/models
```
Should return: `{"object":"list","data":[{"id":"model-name",...}]}`

### **Step 2: Configure Client**
- Use exact URL format for that client (see above)
- Test connection in client settings

### **Step 3: Use AI**
- Send messages through client
- Model on Host PC processes requests
- Responses sent back to Client

---

## 📊 ARCHITECTURE

```
Client PC (Witsy/Cherry)
    ↓ HTTP Request
OpenAI Proxy (Port 8081)
    ↓ Forward
LLAMA.CPP Server (Port 8080)
    ↓ Process
AI Model (GGUF file)
    ↓ Response
Back to Client
```

**Key Points:**
- Proxy accepts multiple concurrent clients
- LLM processes one request at a time (queued)
- All clients share the same model instance
- CORS enabled for browser-based clients

---

## 🔧 TROUBLESHOOTING CHECKLIST

**Connection refused:**
- [ ] Arandu running on Host PC?
- [ ] Model loaded and active?
- [ ] Network Serve activated?
- [ ] Firewall allows port 8081?
- [ ] Using correct IP (not 127.0.0.1)?

**"No models" error:**
- [ ] Model actually running? (check terminal)
- [ ] Proxy can reach llama.cpp on port 8080?
- [ ] Try: `curl http://localhost:8081/v1/models` on Host

**Client sees port but no response:**
- [ ] Wrong URL format for that client?
- [ ] Witsy needs `/v1/` with trailing slash
- [ ] Cherry needs NO `/v1` and NO trailing slash

**Works locally but not across LAN:**
- [ ] Using LAN IP, not localhost?
- [ ] Firewall blocking on Host PC?
- [ ] Both PCs on same network?

---

## ✅ VERIFIED CONFIGURATIONS

**Tested and Working:**
- ✅ Witsy → Arandu across LAN
- ✅ Cherry Studio → Arandu across LAN
- ✅ Multiple clients → Same model
- ✅ curl → Arandu API
- ✅ Python OpenAI SDK → Arandu

**Network Types Tested:**
- ✅ Local LAN (10.0.0.x)
- ✅ Same PC (127.0.0.1)

---

## 📦 FILES NEEDED

**Host PC:**
- `Arandu_0.5.5-beta_x64-portable-CORS.zip`
- GGUF model file(s)

**Current main-branch build outputs (kept for this snapshot):**
- `backend/target/release/bundle/msi/Arandu_0.5.5-1_x64_en-US.msi`
- `backend/target/release/bundle/nsis/Arandu_0.5.5-1_x64-setup.exe`

**Client PCs:**
- Witsy or Cherry Studio AI
- No other requirements

---

## 🎉 RESULT

**Fully functional distributed AI system:**
- One PC runs the AI model
- Multiple PCs can use it simultaneously
- OpenAI-compatible API
- Works with existing tools (Witsy, Cherry, Python SDK)
- No cloud required - completely local

---

**Last Updated:** 2025-02-23
**Tested By:** User
**Status:** PRODUCTION READY

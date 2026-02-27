# Arandu LAN Connection - Critical Findings

**Date:** 2025-02-23
**Session:** Major breakthrough in LAN connectivity

---

## 🔍 DISCOVERY

Each client program requires **drastically different URL formats** for LAN connections, even when connecting to the same Arandu instance.

---

## ✅ CONFIRMED WORKING URLS

### **Witsy**
```
http://10.0.0.119:8084/v1/
```
**Must have:** `/v1/` with trailing slash `/`

### **Cherry Studio AI**
```
http://10.0.0.106:8086
```
**Must NOT have:** `/v1` or trailing slash

---

## ❌ WHAT WAS TRIED AND FAILED

**Initially thought both used same format:**
- ❌ Both using `/v1/` - Cherry breaks
- ❌ Both without `/v1` - Witsy breaks  
- ❌ Both with trailing slash - Cherry breaks
- ❌ Both without trailing slash - Witsy breaks

**The solution:** Each client has unique requirements that must be followed exactly.

---

## 🧪 TESTING METHOD

1. Start with curl to verify network works:
   ```bash
   curl http://10.0.0.106:8081/v1/models
   ```

2. If curl works but client doesn't → URL format issue

3. Try each format variation systematically:
   - With/without `/v1`
   - With/without trailing `/`
   - Different combinations

4. Once found, document exact format immediately

---

## 💡 KEY INSIGHT

**The /v1 suffix handling is client-specific, not standard:**

- Witsy: Expects OpenAI standard `/v1/` path
- Cherry Studio: Handles API versioning internally
- This is why the same URL doesn't work for both

---

## 📊 COMPARISON

| Client | /v1 | Trailing / | Example IP | Example Port |
|--------|-----|------------|------------|--------------|
| Witsy | ✅ | ✅ | 10.0.0.119 | 8084 |
| Cherry | ❌ | ❌ | 10.0.0.106 | 8086 |

Both tested and confirmed working on 2025-02-23.

---

**Related:** arandu-corrected-client-config.md (detailed configuration guide)

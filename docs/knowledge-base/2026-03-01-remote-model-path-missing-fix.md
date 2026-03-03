# Remote Model Launch Path Missing - 2026-03-01

## Issue
Remote models clicking failed with HTTP 422 (Unprocessable Entity)

Error messages:
- "Failed to load resource: the server responded with a status of 422 (Unprocessable Entity)"
- "Launch request failed: Unexpected token 'F', "Failed to "... is not valid JSON"

Console log showed:
```javascript
Remote model clicked: {
  model: {
    id: 'LFM2-2.6B-Exp.Q8_0',
    name: 'LFM2-2.6B-Exp.Q8_0',
    // No 'path' field!
  }
}
```

## Root Cause Chain

### 1. Missing Field in API Response
The `/v1/models/arandu` endpoint only returned model names, not full file paths.

**Original response structure:**
```json
{
  "data": [{
    "id": "LFM2-2.6B-Exp.Q8_0",
    "name": "LFM2-2.6B-Exp.Q8_0",
    "object": "model",
    "owned_by": "arandu",
    // No 'path' field
  }]
}
```

### 2. Frontend Sending Undefined
In `desktop.js` line 5113:
```javascript
terminalManager.openNativeChatForServer(modelName, peerIp, peerPort, model.path);
```

Since `model.path` was undefined, request became:
```json
{
  "model_path": undefined,
  "server_host": "10.0.0.106",
  "server_port": 8081
}
```

### 3. Server Rejected Empty Path
Rust's `Json()` deserializer failed on `undefined`, causing 422 error.

## Fixes Applied

### Fix 1: Add path field to API response structures

**File:** `backend/src/openai_types.rs`
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_gb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,  // ← NEW FIELD
}
```

**File:** `backend/src/openai_proxy.rs` - list_models_arandu()
```rust
let models: Vec<ModelInfo> = scanned_models
    .into_iter()
    .map(|model| ModelInfo {
        id: model.name.clone(),
        object: "model".to_string(),
        created: model.date,
        owned_by: "arandu".to_string(),
        size_gb: Some(model.size_gb),
        quantization: Some(model.quantization),
        architecture: Some(model.architecture),
        date: Some(model.date),
        path: Some(model.path.clone()),  // ← NEW FIELD
    })
    .collect();
```

### Fix 2: Propagate path in Discovery

**File:** `backend/src/discovery.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteModel {
    pub id: String,
    pub name: String,
    pub object: String,
    pub owned_by: String,
    pub instance_id: String,
    pub instance_hostname: String,
    pub api_endpoint: String,
    pub size_gb: Option<f64>,
    pub quantization: Option<String>,
    pub architecture: Option<String>,
    pub date: Option<i64>,
    pub path: Option<String>,  // ← NEW FIELD
}

impl RemoteModel {
    fn from_openai_model(...) -> Self {
        Self {
            // ... other fields ...
            path: model.path,  // ← NEW
        }
    }
}
```

### Fix 3: Update frontend to use path with fallbacks

**File:** `frontend/desktop.js`
```javascript
handleRemoteModelClick(model, peer) {
    // ... validation code ...
    const modelPath = model.path || model.name || model.id;
    terminalManager.openNativeChatForServer(modelName, peerIp, peerPort, modelPath);
    // ...
}
```

### Fix 4: Add missing GgufFileInfo struct

**File:** `backend/src/models.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufFileInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub quantization_type: String,
}
```

**File:** `backend/src/huggingface.rs` - Changed option handling
```rust
let quantization_type = extract_quantization_type(file_path).unwrap_or_else(|| "Unknown".to_string());
```

## Post-Fix Behavior

**API Response Now Includes Full Paths:**
```json
{
  "data": [{
    "id": "LFM2-2.6B-Exp.Q8_0",
    "name": "LFM2-2.6B-Exp.Q8_0",
    "path": "C:\\AI\\models\\LFM2-2.6B-Exp.Q8_0.gguf",
    "size_gb": 2.68,
    "quantization": "Q8_0",
    "architecture": "some_arch"
  }]
}
```

**Client Send Success:**
```json
{
  "model_path": "C:\\AI\\models\\LFM2-2.6B-Exp.Q8_0.gguf",
  "server_host": "10.0.0.106",
  "server_port": 8081
}
```

**Server Response:**
```json
{
  "success": true,
  "message": "Model launched successfully",
  "process_id": "some-uuid",
  "server_host": "127.0.0.1",
  "server_port": 8080
}
```

## Files Modified
- `backend/src/openai_types.rs` - Added path field to ModelInfo
- `backend/src/openai_proxy.rs` - Include path in list_models_arandu response
- `backend/src/discovery.rs` - Added path to RemoteModel, propagate in from_openai_model
- `backend/src/models.rs` - Added GgufFileInfo struct
- `backend/src/huggingface.rs` - Fixed extract_quantization_type return type
- `frontend/desktop.js` - Updated handleRemoteModelClick to use model.path

## Related
- Build: `backend/target/release/Arandu.exe` (Mar 1, 2026)
- Installers: `MSI` and `NSIS` at `backend/target/release/bundle/`
- Issue: Remote models still showing JSON parse error (white screen on remote PC)
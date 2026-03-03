# Arandu User Manual

Complete guide for using the Arandu LLM Desktop Application.

## Table of Contents

- [Introduction](#introduction)
- [Installation & Setup](#installation--setup)
- [Main Interface](#main-interface)
- [Managing Models](#managing-models)
- [HuggingFace Integration](#huggingface-integration)
- [Chat Interface](#chat-interface)
- [Chat History](#chat-history)
- [File Attachments](#file-attachments)
- [AI Model Tracker](#ai-model-tracker)
- [Settings & Configuration](#settings--configuration)
- [Troubleshooting](#troubleshooting)

## Introduction

**Arandu** is a desktop application that provides a user-friendly interface for running local Large Language Models (LLMs) using llama.cpp. It eliminates the need to work with command-line tools, manage DLL files, or remember complex launch parameters.

### Key Features

- **One-click model launching** - Run models with a simple double-click
- **Desktop-style interface** - Icons for each model, organized visually
- **HuggingFace integration** - Search and download models directly
- **Chat history management** - Save, load, and manage conversations
- **File attachments** - Send images and documents to multimodal models
- **AI Model Tracker** - Browse trending models from HuggingFace
- **Automatic backend management** - Download and manage llama.cpp binaries
- **Hardware monitoring** - Real-time RAM and VRAM usage

## Installation & Setup

### Prerequisites

1. **Windows 10/11** (64-bit)
2. **NVIDIA GPU** (optional, but recommended for faster inference)
3. **Sufficient disk space** (models range from 1GB to 50GB+)

### First-Time Setup

1. **Download and run** the installer
2. **On first launch**, Arandu will create a configuration folder at:
   ```
   %USERPROFILE%\.Arandu\
   ```
3. **Choose your model directory** - Select where to store your GGUF model files
4. **Download llama.cpp backend** - The app will prompt you to download the llama-server executable

### Configuration Paths

| Location | Purpose |
|----------|---------|
| `~\.Arandu\models\` | Default location for GGUF model files |
| `~\.Arandu\llama.cpp\` | llama-server executables |
| `~\.Arandu\chats\` | Saved chat conversations |
| `~\.Arandu\config.json` | Application settings |

## Main Interface

When you open Arandu, you'll see a desktop-style interface:

### Desktop Elements

- **Model Icons** - Each GGUF file appears as an icon on the desktop
  - Color bar at bottom shows quantization level (1-bit to 32-bit)
  - "GGUF" label displayed on each icon
  - Update indicator (if linked to HuggingFace)
  
- **Dock/Taskbar** - At the bottom:
  - **Home Button** - Return to desktop
  - **HF Search** - Open HuggingFace model search
  - **Tracker** - Browse AI model database
  - **Backend Manager** - Manage llama.cpp versions
  - **Settings** - Configure directories and appearance

- **System Monitor** - Top-right corner shows:
  - Available disk space
  - RAM usage bar
  - VRAM usage bar (if NVIDIA GPU detected)

### View Modes

Click the **Toggle View** button (grid icon) to switch between:
- **Icon Grid** - Visual icons for each model
- **List View** - Vertical list sorted by file size (largest first)

Your preference is saved automatically.

### Right-Click Menu

Right-click on any model icon to access:

- **Launch Model** - Start the model with internal UI
- **Launch External** - Start with llama.cpp's native web UI
- **Properties** - View and edit model settings
- **Open in File Explorer** - Open containing folder
- **Check for Updates** - Verify if newer version available on HuggingFace
- **Link to HuggingFace** - Associate with HF model ID for updates

## Managing Models

### Adding Models

**Method 1: Manual Download**
1. Download GGUF files from HuggingFace or other sources
2. Place them in your models directory (or any subdirectory)
3. Click the **Refresh** button or restart Arandu

**Method 2: HuggingFace Search**
1. Click the **HF Search** button in the dock
2. Search for models (e.g., "llama", "mistral", "qwen")
3. Browse files and click **Download** on GGUF files

### Understanding Quantization

The color bar on each model icon represents the quantization level:

| Color | Bits | Description | Example |
|-------|------|-------------|---------|
| Deep Red | 1-bit | Extreme compression | IQ1_S, IQ1_M |
| Orange-Red | 2-bit | Very compressed | IQ2_XS, Q2_K |
| Orange | 3-bit | Compressed | IQ3_XS, Q3_K |
| Yellow-Orange | 4-bit | Balanced (recommended) | Q4_K_M, Q4_0 |
| Yellow | 5-bit | Good quality | IQ5_K |
| Lime | 6-bit | Better quality | IQ6_K, Q6_K |
| Green | 7-bit | High quality | IQ7_K |
| Teal | 8-bit | Very high quality | Q8_0, Q8_K |
| Blue | 16-bit | Near-original | F16, BF16 |
| Purple | 32-bit | Full precision | F32 |

**Recommendation**: Q4_K_M (yellow-orange) offers the best balance of quality and speed for most use cases.

### Model Properties

Right-click a model → **Properties** to configure:

- **Custom Arguments** - Additional llama-server parameters
- **Server Host/Port** - Network configuration
- **Presets** - Save different argument combinations
- **HuggingFace Link** - Associate for update checking

### Update Indicators

Models linked to HuggingFace show status indicators:

- **? (Gray)** - Not linked (click to link)
- **✓ (Green)** - Up to date
- **✗ (Red)** - Update available on HuggingFace
- **! (Red)** - Error checking updates
- **⟳ (Spinning)** - Checking in progress

## HuggingFace Integration

### Searching for Models

1. Click **HF Search** in the dock
2. Enter search terms (e.g., "llama 3", "mistral instruct")
3. Browse results with filters:
   - **GGUF Only** - Show only GGUF-format models
   - **Chinese Models** - Filter to Chinese-language models
   - **Backends** - CUDA, Vulkan, ROCm, CPU support
   - **VRAM Limit** - Filter by GPU memory requirements

### Downloading Models

1. Click on a model name to see available files
2. Click **Download** next to GGUF files
3. Monitor progress in the Download Manager
4. Downloads resume automatically if interrupted

### Direct Link Download

For models not in search results:

1. Go to **HuggingFace → Paste Link** tab
2. Paste any HuggingFace model URL
3. Select files to download
4. Click **Download Selected**

Supported URL formats:
```
https://huggingface.co/username/model-name
https://huggingface.co/username/model-name/tree/main
https://huggingface.co/username/model-name/blob/main/model.gguf
```

## Chat Interface

### Starting a Chat

**Method 1: Double-click** any model icon to launch with internal chat UI

**Method 2: Right-click** → **Launch Model** for more options

### Chat Layout

```
┌─────────────────────────────────────────────────────┐
│ [Model Name]                    [Params] [Settings] │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Chat messages appear here                           │
│                                                     │
│ User: Hello!                                        │
│ Assistant: Hi there! How can I help?                │
│                                                     │
├─────────────────────────────────────────────────────┤
│ [+] [Message input...                    ] [Send]   │
└─────────────────────────────────────────────────────┘
```

### Controls

- **Parameters Button** - Adjust temperature, max tokens, etc.
- **Settings** - Model-specific configuration
- **+ Button** - Attach files (images, PDFs, documents)
- **Send** - Send message (or press Enter)

### Parameters Panel

Click **Parameters** to adjust:

- **Temperature** (0.0 - 2.0) - Creativity vs. determinism
- **Max Tokens** - Maximum response length
- **Top P** - Nucleus sampling
- **Top K** - Top-k sampling
- **Repeat Penalty** - Reduce repetition
- **Context Length** - Maximum conversation memory

Changes take effect on next message.

### Regenerating Responses

Hover over any assistant message to see:
- **Regenerate** - Try again with same prompt
- **Copy** - Copy message to clipboard
- **Delete** - Remove this message

## Chat History

Arandu automatically saves your conversations.

### Accessing Chat History

In the chat interface, look for the **History** panel on the left sidebar, or click the **History** button to open the history view.

### History Features

- **New Chat** - Start a fresh conversation
- **Click to Load** - Resume any previous chat
- **Auto-titles** - Chats are automatically titled after 4 conversation turns
- **Timestamps** - See when each chat was created/updated
- **Delete** - Remove unwanted chats

### How Auto-Titles Work

1. Start a new chat (untitled)
2. After 4 back-and-forth exchanges
3. The AI generates a concise title based on the conversation
4. Title is saved automatically

### Manual Title Editing

Right-click any chat in the history list:
- **Rename** - Set custom title
- **Delete** - Remove chat permanently

## File Attachments

Send images and documents to multimodal models (vision-capable models).

### Supported File Types

| Type | Extensions | Notes |
|------|------------|-------|
| Images | .jpg, .jpeg, .png, .gif, .webp, .bmp | Best for vision models |
| Documents | .pdf, .docx, .txt | Text extraction |
| Code | .js, .py, .rs, .cpp, etc. | Sent as text |

### How to Attach Files

1. Click the **+** button next to the message input
2. Select files from the file picker
3. Files appear as preview thumbnails
4. Type your message (optional)
5. Click **Send**

### For Vision Models

Vision-capable models (e.g., LLaVA, BakLLaVA) can analyze images:

```
[Attached: screenshot.png]

User: What's in this image?
Assistant: I can see a desktop interface with... 
```

**Note**: Only vision/multimodal models can process images. Standard text-only models will ignore image attachments.

### Removing Attachments

- Click the **×** on any preview thumbnail to remove it
- Or click outside the file picker to cancel

## AI Model Tracker

Browse and track trending AI models from HuggingFace.

### Opening the Tracker

Click the **Tracker** button (robot icon) in the dock.

### Tracker Features

- **Trending Models** - Popular models from HuggingFace
- **Statistics Panel** - Total models, categories, VRAM requirements
- **Filters**:
  - Category: Text, Image, Video, Audio, Coding, Multimodal
  - Chinese models only
  - GGUF format only
  - VRAM limit
  - Backend support (CUDA, Vulkan, ROCm, CPU, Intel)
  - Quantization level
- **Sort Options**: Downloads, Likes, Date, Name, Size

### Refreshing Data

Click **Refresh** to fetch the latest trending models from HuggingFace.

### Exporting

Click **Export** to save the current model list to a JSON file.

## Remote Model Launch

Automatically launch models on remote Arandu servers over your LAN. Allows clients with less powerful hardware to use models running on servers with strong GPUs.

### How Remote Launch Works

When you click a remote model icon:
1. Your client sends a launch request to the remote server
2. Server automatically launches the model (if not already running)
3. Once ready, your client opens a chat window
4. Multiple clients can connect to the same running model simultaneously

### Prerequisites

**Before using remote launch:**

1. **Network Discovery must be enabled** (Settings → Network Discovery)
2. **Both server and client** must be on the same LAN
3. **Discovery port and API port** must be accessible (no firewall blocking)
4. **At least one GGUF model** exists on the server

### Using Remote Models

**Step 1: Enable Discovery on Server**
1. On the powerful machine (Server), go to **Settings → Network Discovery**
2. Check **Enable Network Discovery**
3. Set an **Instance Name** (e.g., "GPU-Station")
4. Note the **API Port** (default: 8081)

**Step 2: Enable Discovery on Client**
1. On your computer (Client), go to **Settings → Network Discovery**
2. Check **Enable Network Discovery**
3. Wait a few seconds for the server to appear in discovered peers

**Step 3: Launch Remote Model**
1. Switch to **Remote Models View** (click ☁️ button in top-right)
2. Find the remote model you want to use
3. **Double-click** the remote model
4. You'll see: **"Requesting model launch..."** toast
5. Once loaded: **"Model ready!"** toast, then chat window opens

### Remote Chat Interface

The remote chat window has:
- **Title**: "Remote Chat - Model Name (ip:port)"
- **Process ID**: Shown in header for tracking
- **Stop Model Button**: Red button to stop the remote model
- **Iframe**: llama-server web UI from the remote server

### Stopping Remote Models

**Method 1: From Chat Window**
- Click the red **Stop Model** button in the window header
- You'll see "Stopping model..." → "Model stopped successfully" toasts

**Method 2: From Server**
- Go to the server machine
- Stop the model normally (terminal close, etc.)

**Note**: Stopping from one client stops it for ALL clients.

### Multiple Clients

**Multiple clients can connect to the same model simultaneously:**

1. Client A launches Model X on Server
2. Client B clicks the same Model X
3. Client B sees "Model ready!" (no re-launch needed)
4. Both clients can send messages
5. Responses alternate between clients

### Troubleshooting Remote Launch

**Problem**: "Failed to launch model" error

**Solutions**:
1. Verify model path exists on server
2. Check server has llama.cpp backend installed
3. Check firewall allows port 8081 (API port)
4. Verify both machines are on same subnet

**Problem**: "Connection refused" in chat iframe

**Solutions**:
1. Model may still be loading (wait 10-30 seconds)
2. Check server has free VRAM for the model
3. Restart the model if loading hung

**Problem**: Can't see remote models list

**Solutions**:
1. Verify discovery is enabled on BOTH machines
2. Check Discovery Debug Log (🐛 icon in dock)
3. Look for `RECV` entries from other machines' IPs
4. If no `RECV` entries, check firewall:
   - Open PowerShell as Administrator on affected machine
   - Run: `New-NetFirewallRule -DisplayName "Arandu Discovery UDP" -Direction Inbound -Protocol UDP -LocalPort 5352 -Action Allow`
   - Replace 5352 with your discovery port if different

**Problem**: Discovery shows only own beacons

**Solutions**:
1. Windows Firewall blocking inbound UDP traffic
2. Apply firewall rule (see previous problem)
3. Check all machines use same discovery port (default 5352)
4. Verify all machines are on same network segment (10.0.0.x, 192.168.1.x, etc.)

**Known Issues (2026-03-01)**

- ⚠️ Remote Chat White Screen: Chat window may open but show only blank white space on the remote PC
- ⚠️ JSON Parse Error: "Launch request failed: Unexpected token 'F'" error may still occur in some cases

### Network Configuration

**Discovery Port (UDP)**:
- Default: 5352
- Purpose: Broadcast beacons between machines
- Firewall rule: Inbound UDP on this port

**API Port (TCP)**:
- Default: 8081
- Purpose: REST API for model launch and model listing
- Firewall rule: Inbound TCP on this port

**Virtual IPs**:
- You may see 172.x.x.x addresses in debug logs
- These are WSL2/Hyper-V/Docker virtual adapters
- They don't affect LAN discovery and can be ignored
2. Wait 10-15 seconds for UDP beacons to propagate
3. Check both devices are on same network (not guest Wi-Fi)
4. Disable VPN if active

**Problem**: Stop button doesn't work

**Solutions**:
1. Wait a moment (sometimes takes a few seconds)
2. Try stopping from the server machine directly
3. If model is already stopped, button will show "Failed to stop" (this is OK)

### API Endpoints (For Developers)

If you want to programmatically control remote models:

**Launch Model:**
```bash
POST http://SERVER_IP:8081/api/models/launch
{
  "model_path": "/path/to/model.gguf",
  "server_host": "192.168.1.100",
  "server_port": 8081
}
```

**List Active Models:**
```bash
GET http://SERVER_IP:8081/api/models/active
```

**Stop Model:**
```bash
POST http://SERVER_IP:8081/api/models/stop
{
  "process_id": "uuid-string"
}
```

### Security Notes

- Remote launch works ONLY on local area network (LAN)
- Not exposed to internet (binds to 0.0.0.0 but behind NAT/firewall)
- No authentication required (trusted LAN devices)
- Model data stays on server (client only accesses via HTTP API)

### Performance Tips

- Connect via Ethernet instead of Wi-Fi for better latency
- Place server and client on same network switch if possible
- Large responses may take longer over slow connections
- Consider using smaller models for remote access

---

## Settings & Configuration

### Opening Settings

Click the **Settings** button (gear icon) in the dock.

### Configuration Options

**Directories:**
- **Models Directory** - Primary location for GGUF files
- **Additional Directories** - Up to 2 extra model locations
- **Executables Folder** - Where llama.cpp binaries are stored

**Appearance:**
- **Theme Color** - UI accent color
- **Background** - Desktop background style
- **Sync Theme** - Match background to UI theme

**Available Themes:**
- Navy (default)
- Dark Gray
- Purple
- Green
- Red
- Orange
- Blue
- Pink

### Backend Management

Access via **Settings → Llama.cpp Manager** or the **Backend Manager** button:

### Network Discovery

**Settings → Network Discovery** - Share models with other Arandu instances on your LAN.

**Enabling Discovery:**
1. Check **Enable Network Discovery**
2. Set your **Instance Name** (e.g., "Office-PC")
3. Choose **Discovery Port** (default: 5353)
4. Set **Broadcast Interval** (default: 5 seconds)
5. Your **Instance ID** is auto-generated (read-only)

**Viewing Discovered Instances:**
- The table shows other Arandu PCs on your network
- Columns: Status (online/offline), Hostname, IP, Models count
- Click the refresh button to update the list

**Using Remote Models:**
1. Switch to **List View** (icon in top-right of desktop)
2. Left panel: Your local models
3. Right panel: Models from other PCs
4. Click any remote model to connect and chat

**Notes:**
- Discovery only works on your local network (LAN)
- Both PCs must have discovery enabled
- Remote models show "Remote: Hostname" in chat window
- Uses UDP port 5353 (or your chosen port)

1. **View Installed Versions** - See all downloaded backends
2. **Download New Version** - Get latest llama.cpp release
3. **Select Active Version** - Choose which backend to use
4. **Backend Types** - CUDA, ROCm, Vulkan, CPU variants

### Changing Directories

1. Click **Browse** next to any directory field
2. Select new folder in the file dialog
3. Click **Save Configuration**
4. Arandu will scan the new locations

**Note**: Moving your models directory doesn't move existing files. Copy files manually if needed.

## Troubleshooting

### Model Won't Launch

**Problem**: Clicking model does nothing or shows error

**Solutions**:
1. Check that llama.cpp backend is installed (Settings → Backend Manager)
2. Verify backend matches your hardware (CUDA for NVIDIA, CPU for others)
3. Check model file isn't corrupted (try re-downloading)
4. Check logs in the terminal window for specific errors

### Out of Memory Errors

**Problem**: "CUDA out of memory" or system freezes

**Solutions**:
1. Use a smaller model (lower quantization or fewer parameters)
2. Reduce context length in model properties
3. Close other GPU-intensive applications
4. Use CPU backend instead of CUDA

### Chat History Not Loading

**Problem**: Previous chats don't appear

**Solutions**:
1. Check `~\.Arandu\chats\` folder exists
2. If `index.json` is corrupted, it will be reset automatically
3. Chat files (.md) are still preserved even if index is reset

### Downloads Failing

**Problem**: Downloads stop or fail partway

**Solutions**:
1. Check internet connection
2. Resume download (progress is saved)
3. Check disk space availability
4. Try direct download from HuggingFace website

### Slow Performance

**Problem**: Model responses are very slow

**Solutions**:
1. Use GPU backend (CUDA for NVIDIA)
2. Use lower quantization (Q4_K_M is faster than Q8_0)
3. Reduce context length
4. Close other applications
5. Ensure GPU drivers are up to date

### Can't Find Downloaded Models

**Problem**: Downloaded models don't appear on desktop

**Solutions**:
1. Click **Refresh** button
2. Check models are in GGUF format (other formats not supported)
3. Verify correct models directory in Settings
4. Check Additional Directories if using multiple locations

### Update Check Not Working

**Problem**: Update indicators show "!" or don't appear

**Solutions**:
1. Right-click model → **Link to HuggingFace** to set association
2. Check internet connection
3. HuggingFace API rate limit may be exceeded (wait 1 hour)

### File Attachments Not Working

**Problem**: Files don't send or model ignores them

**Solutions**:
1. Ensure you're using a vision/multimodal model
2. Check file type is supported (images, PDFs, text)
3. Large files may take time to process
4. Some models have file size limits

### Settings Not Saving

**Problem**: Changes to settings don't persist

**Solutions**:
1. Click **Save Configuration** button
2. Check permissions on `~\.Arandu\` folder
3. Ensure config.json isn't read-only
4. Try running Arandu as administrator

### Getting Help

If issues persist:

1. Check the **System Monitor** for resource usage
2. Look at terminal/log output for error messages
3. Visit the GitHub repository: https://github.com/fredconex/Arandu
4. Check for updates to Arandu

## Tips & Best Practices

### Performance Optimization

- **Use Q4_K_M quantization** for best speed/quality balance
- **GPU layers** - Models offload to GPU automatically when possible
- **Context length** - Lower values use less VRAM
- **Batch size** - Adjust based on your hardware

### Model Selection

- **General use**: Llama 3, Mistral, Qwen 2.5
- **Coding**: CodeLlama, DeepSeek-Coder, Qwen-Coder
- **Vision**: LLaVA, BakLLaVA (for image understanding)
- **Small/ Fast**: Phi, Gemma 2B, Qwen 0.5B

### Organization

- Create subfolders in your models directory
- Use descriptive filenames
- Link models to HuggingFace for easy updates
- Delete old chats regularly to save disk space

### Safety

- Models run locally - data stays on your computer
- Be cautious with downloaded models (use trusted sources)
- Keep Windows and GPU drivers updated
- Regular backups of important chats

---

**Version**: 0.5.5-beta  
**Last Updated**: February 2026  
**Homepage**: https://github.com/fredconex/Arandu

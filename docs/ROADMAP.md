# Arandu Future Roadmap

**Version:** 0.5.5-beta  
**Last Updated:** 2025-02-23  
**Status:** Task 6 Complete, Tasks 7-8 Planned

---

## ✅ COMPLETED (Current State)

### **Phase 1: Core Infrastructure** ✅ DONE
- ✅ OpenAI-compatible API proxy server
- ✅ Chat completions with streaming (SSE)
- ✅ Multiple client support (Witsy, Cherry AI, Python SDK)
- ✅ Cross-LAN connectivity with CORS
- ✅ Network auto-detection (finds LAN IP automatically)
- ✅ Concurrent client connections
- ✅ Non-streaming and streaming responses
- ✅ Error handling with proper HTTP status codes
- ✅ Health check endpoint
- ✅ Model listing endpoint
- ✅ Portable distribution (ZIP)
- ✅ Windows installer (NSIS)

### **Phase 2: Client Compatibility** ✅ DONE
- ✅ Witsy integration working
- ✅ Cherry Studio AI integration working
- ✅ Python OpenAI SDK working
- ✅ curl/HTTP clients working
- ✅ LAN connection guide documented
- ✅ Client-specific URL formats documented

### **Phase 3: Multi-Model Support** ✅ DONE
- ✅ Launch multiple models (different ports)
- ✅ Model process management
- ✅ Auto port assignment (8080, 8081, 8082...)
- ✅ Process tracking and cleanup
- ✅ Model configuration per-file

---

## 🚧 PLANNED (Next Steps)

### **Phase 4: Audio Support (Task 7)** ⏳ NOT STARTED
**Goal:** Add speech-to-text and text-to-speech capabilities

**Endpoints to Implement:**
- `POST /v1/audio/transcriptions` - Speech-to-text via whisper.cpp
- `POST /v1/audio/speech` - Text-to-speech

**Technical Requirements:**
- [ ] Create `backend/src/whisper_manager.rs`
- [ ] WhisperServer struct for managing whisper.cpp process
- [ ] Download whisper.cpp backend (like llama.cpp)
- [ ] Audio file upload handling
- [ ] Transcription request forwarding
- [ ] TTS response generation
- [ ] Frontend integration (audio widgets)

**Dependencies:**
- whisper.cpp binaries
- Audio file format support (mp3, wav, etc.)

**Estimated Effort:** 2-3 weeks

---

### **Phase 5: Image Generation (Task 8)** ⏳ NOT STARTED
**Goal:** Add text-to-image generation via Stable Diffusion

**Endpoints to Implement:**
- `POST /v1/images/generations` - Generate images from text

**Technical Requirements:**
- [ ] Create `backend/src/image_manager.rs`
- [ ] ImageGenerationServer struct
- [ ] Stable Diffusion backend integration
- [ ] Image file handling and storage
- [ ] Generation queue management
- [ ] Frontend image gallery

**Dependencies:**
- Stable Diffusion backend (ComfyUI, Automatic1111, or native)
- Image storage system
- GPU VRAM management (images need more VRAM than text)

**Estimated Effort:** 3-4 weeks

---

### **Phase 6: Advanced Features** 💡 PROPOSED

#### **6.1 Model Router / Auto-Switching** 💡
**Problem:** Currently only one model active at a time per proxy
**Solution:** Intelligent routing based on model name in request

**Implementation:**
- [ ] Track multiple running models
- [ ] Route requests to correct model based on "model" parameter
- [ ] Load balance between identical models
- [ ] Queue management per model

**Benefits:**
- Use different models for different tasks simultaneously
- Small model for quick tasks, large model for complex tasks
- No manual stopping/starting

**Estimated Effort:** 2-3 weeks

#### **6.2 VRAM Optimization** 💡
**Problem:** Models consume VRAM even when idle
**Solution:** Dynamic loading/unloading

**Implementation:**
- [ ] Monitor VRAM usage
- [ ] Auto-unload idle models after timeout
- [ ] Smart model caching (LRU)
- [ ] VRAM usage dashboard

**Estimated Effort:** 1-2 weeks

#### **6.3 Preset Management System** 💡
**Problem:** Manual argument configuration per model
**Solution:** Save/load presets

**Implementation:**
- [ ] Create preset system (JSON-based)
- [ ] Preset UI in frontend
- [ ] Default presets (Fast, Quality, Balanced)
- [ ] Import/export presets
- [ ] Per-model preset memory

**Estimated Effort:** 1 week

#### **6.4 Auto-Updater** 💡
**Problem:** Manual download of new versions
**Solution:** Built-in update system

**Implementation:**
- [ ] Check for updates on startup
- [ ] Download and install updates
- [ ] Changelog display
- [ ] Rollback capability

**Estimated Effort:** 1 week

---

### **Phase 7: Platform Expansion** 💡 FUTURE

#### **7.1 Linux Support** 💡
**Current:** Windows only
**Goal:** Native Linux support

**Requirements:**
- [ ] Linux-specific process management
- [ ] Linux llama.cpp binaries
- [ ] Testing on Ubuntu, Fedora, Arch

**Estimated Effort:** 2-3 weeks

#### **7.2 macOS Support** 💡
**Goal:** macOS support (Intel and Apple Silicon)

**Requirements:**
- [ ] macOS process management
- [ ] Metal GPU support for M1/M2/M3
- [ ] Universal binary

**Estimated Effort:** 3-4 weeks

---

## 📊 PRIORITY MATRIX

### **High Priority (Next 1-2 months)**
1. **Task 7: Audio Support** - Most requested feature
2. **Task 8: Image Generation** - Differentiates from competitors
3. **Bug Fixes** - Based on user feedback

### **Medium Priority (3-6 months)**
4. **Model Router** - Power user feature
5. **VRAM Optimization** - Performance improvement
6. **Preset Management** - UX improvement

### **Low Priority (6+ months)**
7. **Linux Support** - Platform expansion
8. **macOS Support** - Platform expansion
9. **Auto-Updater** - Nice to have

---

## 🎯 IMMEDIATE NEXT STEPS

### **What to do next:**

**Option 1: Implement Audio (Task 7)**
- Most practical next step
- Adds significant value
- Clear implementation path

**Option 2: Implement Image Generation (Task 8)**
- More complex but impressive
- Requires more VRAM management
- Larger user impact

**Option 3: Polish Current Features**
- Fix any remaining bugs
- Improve documentation
- Add user-requested enhancements

**Option 4: Model Router**
- Technical challenge
- Power user feature
- Foundation for future scaling

---

## 📈 SUCCESS METRICS

**Current:**
- ✅ Text generation working
- ✅ Multiple clients supported
- ✅ LAN connectivity stable
- ✅ Portable distribution ready

**Milestones:**
- 🎯 Audio support (Task 7) - Increase utility by 40%
- 🎯 Image generation (Task 8) - Differentiation feature
- 🎯 Model router - Enable advanced workflows
- 🎯 Cross-platform - Expand user base by 60%

---

## 🤔 RECOMMENDATION

**Start with Task 7 (Audio Support)** because:
1. **Clear implementation path** - Similar to existing llama.cpp integration
2. **High user value** - Speech-to-text is commonly requested
3. **Manageable scope** - 2-3 weeks vs 3-4 weeks for images
4. **Foundation building** - Establishes pattern for multi-backend support

**Then Task 8 (Images)** - after audio is stable

**Then Model Router** - after core features complete

---

**What would you like to tackle next?**
- Audio support (Task 7)?
- Image generation (Task 8)?
- Model router (advanced feature)?
- Something else entirely?

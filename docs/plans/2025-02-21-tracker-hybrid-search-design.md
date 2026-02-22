# Tracker Hybrid Search Design

## Overview
Implement a hybrid local + live search system for the AI Model Tracker that shows cached results immediately while fetching fresh data from HuggingFace in the background.

## Design Sections

### 1. Data Flow & Lifecycle
- **On tracker open:**
  1. Load cached models from SQLite and render immediately
  2. Kick off automatic background live fetch using current filters
  3. Merge live results into grid, tagging new/updated models
- **Manual refresh:** Full cache refresh + DB save, then update live overlay

### 2. UI/UX Changes - Total Models Badge
Sticky header above models grid showing:
- Left: "📦 Total cached: X models" (from SQLite stats)
- Right: "🔍 Live results: Y models" (updates when live fetch completes)
- Unified: "📦 X models found" when live completes
- Real-time updates as filters change (cached instant, live with spinner)

### 3. Live Fetch Mechanism & Merge Logic

#### Backend
- Add `get_tracker_live_results(filters)` Tauri command
- Calls HF API with filter params (query, categories, etc.)
- Returns `Vec<TrackerModel>` without DB save (pure live data)
- Limit: 100-200 models for speed

#### Frontend Merge Strategy
1. Show cached models immediately
2. Background fetch with `get_tracker_live_results()`
3. Merge on arrival:
   - Compare by model ID
   - Add new models (tag with "🆕 New" badge)
   - Update existing with fresh stats
   - Keep cached-only visible (tagged "📦 Cached")
4. Stats update showing both counts

#### Conflict Resolution
- Live data wins for display (doesn't overwrite DB)
- "Sync to Cache" button persists live results

### 4. API Efficiency & Rate Limiting
- Debounce live fetch: 500ms after user stops changing filters
- Session cache: 30-second memory cache for unchanged filters
- Concurrent limit: 1 live fetch at a time (cancel previous on change)
- Default fetch size: 100 models
- User toggle: "🔍 Live search" on/off

#### Rate Limit Awareness
- Track HF API calls per minute (60 unauthenticated limit)
- Warning at 50 calls: "⚠️ Approaching limit"
- Auto fallback to cached-only mode when limit hit

## Success Criteria
- [ ] Models badge shows cached + live counts
- [ ] Tracker opens instantly with cached data
- [ ] Live results merge seamlessly in background
- [ ] Rate limiting prevents API exhaustion
- [ ] User can disable live search
- [ ] Missing models are detected and shown

## Related Files
- `backend/src/tracker_scraper.rs` - HF API integration
- `backend/src/tracker_manager.rs` - SQLite caching
- `backend/src/lib.rs` - Tauri commands
- `frontend/modules/tracker-app.js` - UI logic
- `frontend/css/tracker.css` - Styling

## Date: 2025-02-21
## Status: Approved for Implementation

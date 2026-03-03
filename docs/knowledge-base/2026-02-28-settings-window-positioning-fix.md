# Settings Window Positioning Fix - 2026-02-28

## Issue
Settings window was appearing huge and extending outside the visible app area, making it impossible to see all content.

## Root Cause
The settings window only centered on first open (when `left` style was not set). If the window was previously positioned outside the viewport or the window was resized, it wouldn't recenter properly. Additionally, there were no viewport constraints to keep the window within bounds.

## Solution

### 1. Modified `toggleSettingsPanel()` in `frontend/desktop.js`
Changed from conditional centering to always center and constrain:

**Before:**
```javascript
// Center the window if it's the first time opening
if (!windowElement.style.left || windowElement.style.left === 'auto') {
    windowElement.style.left = '50%';
    windowElement.style.top = '50%';
    windowElement.style.transform = 'translate(-50%, -50%)';
}
```

**After:**
```javascript
// Always center and constrain the window within viewport
this.centerAndConstrainWindow(windowElement);
```

### 2. Added `centerAndConstrainWindow()` method in `frontend/desktop.js`
New method that:
- Calculates viewport dimensions
- Gets window dimensions
- Sets max-width/height to 90% of viewport
- Calculates centered position
- Ensures window stays within viewport with 20px padding
- Removes transform (uses absolute positioning)
- Applies position with bounds checking

```javascript
centerAndConstrainWindow(windowElement) {
    // Get viewport dimensions
    const viewportWidth = window.innerWidth || document.documentElement.clientWidth;
    const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
    
    // Get window dimensions
    const windowWidth = windowElement.offsetWidth || 750;
    const windowHeight = windowElement.offsetHeight || 464;
    
    // Calculate maximum allowed dimensions (90% of viewport)
    const maxWidth = Math.min(750, viewportWidth * 0.9);
    const maxHeight = Math.min(600, viewportHeight * 0.9);
    
    // Apply max dimensions
    windowElement.style.maxWidth = `${maxWidth}px`;
    windowElement.style.maxHeight = `${maxHeight}px`;
    
    // Calculate centered position
    let left = (viewportWidth - Math.min(windowWidth, maxWidth)) / 2;
    let top = (viewportHeight - Math.min(windowHeight, maxHeight)) / 2;
    
    // Ensure window stays within viewport bounds with padding
    const padding = 20;
    left = Math.max(padding, Math.min(left, viewportWidth - maxWidth - padding));
    top = Math.max(padding, Math.min(top, viewportHeight - maxHeight - padding));
    
    // Apply position
    windowElement.style.left = `${left}px`;
    windowElement.style.top = `${top}px`;
    windowElement.style.transform = 'none';
    
    // Ensure window is visible
    windowElement.style.display = 'flex';
}
```

### 3. Updated CSS in `frontend/css/windows.css`
Added viewport constraints and overflow handling:

```css
.settings-window {
    width: 750px;
    min-height: 464px;
    height: 464px;
    max-width: 90vw;          /* NEW: Prevent overflow */
    max-height: 90vh;         /* NEW: Prevent overflow */
    position: fixed;          /* NEW: Keep in viewport */
    overflow: hidden;         /* NEW: Handle overflow */
}

.settings-window .window-content {
    overflow-y: auto;         /* NEW: Scroll if needed */
    max-height: calc(90vh - 50px);  /* NEW: Constrain content */
}
```

## Files Modified
- `frontend/desktop.js` - Added centerAndConstrainWindow method, updated toggleSettingsPanel
- `frontend/css/windows.css` - Added max constraints and overflow handling

## Build
- Command: `cargo build --release`
- Duration: 3m 07s
- Output: `target/release/Arandu.exe` (11MB)
- Timestamp: 2026-02-28 11:15
- Status: ✅ SUCCESS

## Behavior After Fix
1. Settings window always centers when opened
2. Window cannot exceed 90% of viewport width/height
3. Window maintains 20px padding from viewport edges
4. Content scrolls if window is too small
5. Window stays fully visible regardless of previous position

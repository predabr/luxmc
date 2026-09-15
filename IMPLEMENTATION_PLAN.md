# Luxmc Launcher v1.7.1+ — Implementation Plan

## Overview
Critical RAM leak (+1 GB every 2-3s) + UI/UX fixes. Plan organized by priority and file.

---

## Phase 1 — Critical Memory Leak Fixes (P0)

### 1.1 `src/lib/components/ui/ResourceMonitor.svelte`
**Problem**: `$effect` reads/writes `lastRunning` + `pollHandle` ($state) → infinite re-render loop at 1Hz. `readings = [...readings, sample]` allocates new array every tick.

**Fix**:
- Replace `$effect` with `onMount`/`onDestroy` for interval management
- Use fixed-size circular buffer (pre-allocated array of 60) instead of spread+slice
- Store index pointer instead of mutating array reference

```svelte
<!-- BEFORE (lines 36-47) -->
$effect(() => {
  const shouldRun = running && pid !== null;
  if (shouldRun && !lastRunning) {
    pollHandle = window.setInterval(() => tick(pid), 1000);
    lastRunning = true;
  }
  if (!shouldRun && lastRunning) {
    clearInterval(pollHandle!);
    pollHandle = null;
    lastRunning = false;
  }
});

<!-- AFTER -->
let idx = $state(0);
const MAX_READINGS = 60;
let readings = $state<Reading[]>(Array(MAX_READINGS).fill(null).map(() => ({ cpuPercent: 0, memMb: 0, ts: 0 })));

onMount(() => {
  if (running && pid !== null) {
    pollHandle = window.setInterval(() => tick(pid), 1000);
  }
});

onDestroy(() => {
  if (pollHandle !== null) clearInterval(pollHandle);
});

async function tick(pid: number | null) {
  if (!pid) return;
  try {
    const sample = await sampleProcess(pid);
    if (sample) {
      readings[idx] = sample;
      idx = (idx + 1) % MAX_READINGS;
    }
  } catch {}
}
```

**Derived updates**: `current` reads `readings[idx === 0 ? MAX_READINGS - 1 : idx - 1]`, `max` iterates fixed array.

---

### 1.2 `src/routes/+layout.svelte`
**Problems**:
- Line 47-49: `$effect` sets toast instance → runs on every `toastsInstance` change
- Line 238-243: `$effect` calls `applyAdaptivePalette` on every active profile change (heavy DOM work)
- Line 245-249: `$effect` stops soundscape on `isGameRunning` — fine but can be `onMount` cleanup
- Line 252-304: `$effect` with `rpcTimeout` — clears/sets timeout on every route/settings change (OK)
- Line 306-323: Konami code listener — OK (cleanup returns fn)
- Line 324-332: Client keys listener — OK

**Fixes**:
- Move toast binding to `onMount` (single set)
- Debounce `applyAdaptivePalette` (300ms) or move to profile store subscriber
- Keep RPC $effect but ensure `rpcTimeout` is properly cleared (already done)

---

### 1.3 `src/lib/components/ui/VibrantAccent.svelte`
**Problem**: Line 40-46 `$effect` calls async `extractColor` on every `src` change. `node-vibrant` is heavy; rapid src changes = memory pressure.

**Fix**:
- Debounce `src` changes (300ms)
- Cache last processed URL to avoid re-processing same image
- Abort previous extraction if new src arrives

```svelte
let lastSrc = $state("");
let abortCtrl = $state<AbortController | null>(null);

$effect(() => {
  if (src === lastSrc) return;
  lastSrc = src ?? "";
  
  if (abortCtrl) abortCtrl.abort();
  abortCtrl = new AbortController();
  
  if (src) {
    setTimeout(() => extractColor(src, abortCtrl.signal), 300);
  } else {
    resetAccent();
  }
  
  return () => abortCtrl?.abort();
});
```

---

### 1.4 `src/lib/components/ui/SkinViewer3D.svelte`
**Problems**:
- Lines 326-348: Four `$effect` watchers call `updateSkin`/`updateCape`/`viewer.autoRotate`/`viewer.renderPaused` — each prop change triggers re-render + potential skin reload
- `loadAndHealSkin` creates `Image` + `Canvas` + `getImageData`/`putImageData` per call — heavy GC pressure
- No cleanup of in-flight skin loads on prop change/unmount

**Fixes**:
- Consolidate into single `$effect` with `skinUrl`, `slim`, `cape`, `customCapeUrl`, `autoRotate`, `active` as deps
- Add `abortController` for in-flight skin loads
- Memoize healed skin canvas by URL+slim key (simple LRU cache, max 5)
- Ensure `viewer.dispose()` called in `onDestroy` (already present at line 86)

---

### 1.5 Array Re-allocation Patterns (Multiple Files)

| File | Line | Pattern | Fix |
|------|------|---------|-----|
| `src/lib/components/ui/ResourceMonitor.svelte` | 19 | `readings = [...readings, sample].slice(-60)` | Circular buffer (see 1.1) |
| `src/routes/friends/+page.svelte` | 140,141,161,163,236,238,240,263,305,341,342,363,364 | `arr = [...arr, item]` / `arr = [...arr]` | Use `push` + `splice` for fixed-size; or `$state.snapshot` pattern |
| `src/lib/components/ui/Toasts.svelte` | 19 | `toasts = [...toasts, newToast]` | OK (toasts are short-lived, low frequency) |
| `src/routes/skins/+page.svelte` | 464,523 | `const merged = [...savedSkins]` | OK (user-initiated, not polling) |
| `src/routes/instances/[id]/+page.svelte` | 852 | `fileBreadcrumbs = [...fileBreadcrumbs, folderName]` | OK (navigation, not polling) |
| `src/routes/mods/+page.svelte` | 251,269 | `new Set([...set, id])` | Use `set.add(id)` directly on `$state` Set |

**Priority fix**: `friends/+page.svelte` — chat messages accumulate unbounded. Add `.slice(-100)` on every push (already present at 140, 163) but the `friends = [...friends]` reassignment triggers full reactivity. Use mutable `push`/`splice` on the array content instead of replacing reference.

---

### 1.6 Tauri Event Listener Cleanup
**Current state**: `logs/+page.svelte` (lines 15-17, 56-60) correctly stores `unlisten` fns and calls them in `onDestroy`. ✓

**Check needed**: 
- `src/lib/components/ui/DownloadProgressBar.svelte` line 107 — verify `onDestroy` calls `unlisten`
- `src/lib/components/ui/JavaManagerModal.svelte` line 61 — verify cleanup
- `src/routes/mods/+page.svelte` line 235 — verify cleanup

**Action**: Audit each component with `listen()` call; ensure `onDestroy` calls returned unlisten function.

---

### 1.7 WebGL/Heavy Resource Disposal
| Component | Resource | Cleanup Status |
|-----------|----------|----------------|
| `SkinViewer3D.svelte` | `viewer.dispose()` | ✓ (line 86) |
| `LiveWallpaper.svelte` | Three.js scene/renderer | Check `onDestroy` |
| `Cutscene.svelte` | Canvas/WebGL | Check `onDestroy` |
| `AudioVisualizer.svelte` (if exists) | AudioContext/Analyser | Check `onDestroy` |

**Action**: Verify each visual component has `onDestroy` disposing WebGL contexts, cancelling animation frames, removing event listeners.

---

### 1.8 Native Heap Trim (Rust) — `src-tauri/src/commands/optimizer.rs`
**Current**: Lines 69-101 implement `optimizer_trim_memory` with `malloc_trim(0)` (Linux) and `SetProcessWorkingSetSize` (Windows). ✓

**Missing**: 
- Auto-call after game exit (in `listenGameExit` handler in `+layout.svelte` line 178-213)
- Periodic call during long-running launcher (every 5-10 min)

**Fix**: 
- In `+layout.svelte` `listenGameExit` callback (after line 210), add `optimizer_trim_memory()`
- Add periodic timer in `onMount` of `+layout.svelte`: `setInterval(optimizer_trim_memory, 5 * 60 * 1000)`

---

## Phase 2 — UI/UX Fixes (P1)

### 2.1 Remove Fake Friends Data

**Files**:
- `src/lib/components/layout/RightSidebar.svelte` (lines 31-76): Remove static `friends` array (pedro_dev, Lucas_Miner, Kiro_PvP, AnaCraft, GuiForge)
- `src/lib/components/home/FriendsRadarWidget.svelte` (lines 39-79): Remove identical static `friends` array

**Replace with**: 
- Real friends integration (future) OR
- Toggleable "Luxmc News/Feed" section (RSS/JSON from GitHub releases or static JSON)
- For now: show empty state with "Conecte sua conta para ver amigos" + link to `/friends`

---

### 2.2 Remove Duplicate Friends Icon from Left Sidebar
**File**: `src/lib/components/layout/Sidebar.svelte` (lines 166-182)
**Action**: Remove the entire "Friends Route with Tooltip" block (lines 166-182). The Friends page is accessible via `/friends` route; sidebar already has GitHub, Client Suite, Settings at bottom.

---

### 2.3 Restore Premium Left Sidebar Button Styling
**File**: `src/lib/components/layout/Sidebar.svelte`

**Current issues**:
- Active indicator: thin 1px bar (line 90) — should be wider glow pill
- Inactive: `nav-pill-inactive` — lacks elevated bg, subtle border, champagne glow
- Hover transitions: 200ms — should be 150ms

**Target design** (from reference):
- Background: `#141518` (elevated charcoal)
- Border: `rgba(202,169,124,0.25)` (champagne glow)
- Active: filled champagne `#caa97c` with inner glow `shadow-[0_0_10px_#caa97c]`
- Icon color: active `#15171c`, inactive `#8a8d98` → hover `white`
- Transition: `150ms` cubic-bezier

**CSS classes to update** (in global CSS or inline):
```css
.nav-pill-active {
  @apply bg-[#caa97c] text-[#15171c] shadow-[0_0_12px_rgba(202,169,124,0.4)] ring-1 ring-[#d8bc98]/50;
}
.nav-pill-inactive {
  @apply bg-[#141518] border border-white/5 text-[#8a8d98] hover:bg-[#1c1d22] hover:border-[#caa97c]/30 hover:text-white;
}
```

Apply to all nav items (main + dynamic instances + bottom section).

---

### 2.4 Populate Empty Home Page Bottom Section
**File**: `src/routes/+page.svelte`

**Current**: Sections defined in `layoutStore` include `hero`, `quickInstances`, `favoriteServer`, `curatedPacks`, `gamingStats`, `friendsRadar`, `tools`. The "empty dark area" suggests some sections are disabled or not rendering.

**Required widgets to ensure enabled & rendered**:
1. **Death Detector** (Death Coordinates Recovery) — Component: `src/lib/components/instances/DeathDetectorModal.svelte` → Create widget version or integrate into `gamingStats` section
2. **Mini-Player Musical (Jukebox)** — Component: `src/lib/components/ui/MiniPlayer.svelte` (already imported in `+layout.svelte` line 17, rendered line 390) → Ensure visible on Home
3. **News & Patch Notes Mural** — New component: `NewsFeedWidget.svelte` fetching from GitHub releases or static JSON
4. **Quick Screenshots Gallery** — Component: `src/lib/components/home/ScreenshotsWidget.svelte` (check if exists) or reuse `screenshots/+page.svelte` logic

**Action**:
- Verify `layoutStore` default sections have all 7 enabled (they do at lines 15-71)
- Add `NewsFeedWidget` component
- Add `ScreenshotsWidget` component (carousel of recent screenshots)
- Ensure `MiniPlayer` is accessible from Home (already global in layout)
- Integrate `DeathDetector` as widget in `gamingStats` section or new `deathDetector` section

---

## Phase 3 — Verification & Testing

### 3.1 Memory Leak Verification
1. Run `pnpm tauri dev`
2. Open DevTools → Memory tab
3. Take heap snapshot, wait 30s, take snapshot, compare
4. Verify no steady growth in:
   - `Detached DOM nodes`
   - `Array` / `Object` counts from polling components
   - `Canvas` / `WebGLRenderingContext` counts

### 3.2 Performance Checks
- `pnpm check` (svelte-check) — zero errors
- `cargo check` (in `src-tauri/`) — zero errors
- `pnpm tauri build` — successful release bundle

### 3.3 UI Regression Tests
- Left sidebar: hover/active states, tooltips, instance icons
- Home page: all 7 sections render, no empty gaps
- Right sidebar: no fake friends, shows news/empty state
- Friends page (`/friends`): real integration placeholder works
- Skin 3D viewer: rotate, zoom, cape toggle, no leaks on prop change

---

## File Change Summary

| File | Phase | Changes |
|------|-------|---------|
| `src/lib/components/ui/ResourceMonitor.svelte` | P0 | Circular buffer, onMount/onDestroy interval |
| `src/routes/+layout.svelte` | P0 | Toast binding, debounced palette, periodic trim |
| `src/lib/components/ui/VibrantAccent.svelte` | P0 | Debounced extraction, abort controller |
| `src/lib/components/ui/SkinViewer3D.svelte` | P0 | Consolidated effects, skin cache, abort in-flight |
| `src/routes/friends/+page.svelte` | P0 | Mutable array ops for chat messages |
| `src/lib/components/ui/DownloadProgressBar.svelte` | P0 | Verify unlisten cleanup |
| `src/lib/components/ui/JavaManagerModal.svelte` | P0 | Verify unlisten cleanup |
| `src/routes/mods/+page.svelte` | P0 | Verify unlisten cleanup |
| `src/lib/components/layout/RightSidebar.svelte` | P1 | Remove fake friends array |
| `src/lib/components/home/FriendsRadarWidget.svelte` | P1 | Remove fake friends array |
| `src/lib/components/layout/Sidebar.svelte` | P1 | Remove duplicate friends icon, restore premium styling |
| `src/routes/+page.svelte` | P1 | Ensure all sections render, add NewsFeedWidget, ScreenshotsWidget |
| `src/lib/components/home/NewsFeedWidget.svelte` | P1 | **New file** — news/feed component |
| `src/lib/components/home/ScreenshotsWidget.svelte` | P1 | **New file** — screenshot carousel |
| `src-tauri/src/commands/optimizer.rs` | P0 | Verify trim works, add periodic call from frontend |

---

## Execution Order
1. **P0 Memory fixes** (1.1–1.8) — run `pnpm check` + `cargo check` after each file
2. **P1 UI fixes** (2.1–2.4) — visual verification in browser
3. **Verification** (3.1–3.3) — memory profiling + build

---

## Questions for Clarification

1. **News Feed Source**: Should `NewsFeedWidget` fetch from GitHub Releases API, a static JSON hosted on CDN, or a dedicated `/api/news` endpoint? (Recommendation: GitHub Releases — zero infra)
2. **Friends Integration**: Is there a backend/P2P service for real friends status, or should the placeholder remain "Connect account to see friends" indefinitely?
3. **Death Detector**: Should this be a standalone widget or merged into `gamingStats` section?
4. **Sidebar Styling**: Confirm the exact hex values — `#141518` bg, `#caa97c` champagne, `#d8bc98` lighter champagne for active glow?
5. **Periodic Trim Interval**: 5 minutes OK, or prefer 10 minutes?
# Providers Module Redesign Spec (Friendly & Professional)

## 1. Overview
The goal of this redesign is to align the `Providers` management view with the "Friendly Ethereal Frost" design language. It replaces the default Element Plus list and heavy dialogs with a customized, airy, and interactive aesthetic that clearly separates CLI environments (Claude Code, Codex, Gemini) and proxy vs. direct modes.

## 2. Page Structure & Navigation
*   **CLI Tabs (Top):**
    *   Transition from boxed/border-heavy tabs to clean, text-based tabs with an accent blue bottom underline (`--accent-blue`) for the active state.
    *   Generous spacing between tabs (`gap: 32px`).
*   **Page Header & Mode Switch:**
    *   Uses the unified **Segmented Control** for switching between "中转模式" (Proxy Mode) and "官方模式" (Direct Mode). The control has a slightly darker grey background (`#e2e8f0`) framing pure white active pills.
    *   Primary action button ("+ 添加服务商") features the brand's accent blue, a soft drop shadow, and a subtle float interaction (`transform: translateY(-2px)`) on hover.

## 3. Provider List (Card View)
*   **Container:** The list lives inside a unified floating card container (`--card-bg`, `--radius-lg`, `--shadow-soft`) to maintain visual cleanliness.
*   **Row Items:**
    *   Rows are separated by a very light border (`#f1f5f9`), removing traditional heavy table grids.
    *   A subtle background color change (`#f8fafc`) on row hover indicates interactivity.
*   **Status Indicators (Tags):**
    *   Pill-shaped, soft-background tags for status elements to reduce visual noise:
        *   **Success (Mappings):** Emerald Green (`rgba(16, 185, 129, 0.1)` bg, `--success-green` text).
        *   **Warning (Blacklist):** Amber (`rgba(245, 158, 11, 0.1)` bg, `--warning-amber` text).
        *   **Danger (Blocked):** Rose Red (`rgba(244, 63, 94, 0.1)` bg, `--danger-red` text). Blocked provider rows also feature a faint red tint (`rgba(244, 63, 94, 0.02)`) over the entire row for immediate visibility.
        *   **Info (Disabled):** Slate grey (`#f1f5f9` bg, `--text-muted` text) with an overall row opacity of `0.75`.
*   **Actions:**
    *   Replaced default switches with iOS-style toggles (green for active, grey for disabled).
    *   Action buttons ("编辑", "更多") are borderless and use text colors with soft hover backgrounds.

## 4. Add/Edit Provider Modal
*   **Backdrop:** Implements a blurry overlay (`backdrop-filter: blur(4px)`) over the application to focus attention entirely on the modal.
*   **Form Design:**
    *   Fields feature subtle borders that transition to a blue glow (`box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.1)`) upon focus.
    *   Inputs like "失败阈值" and "拉黑时长" are positioned side-by-side using Flexbox to save vertical space.
*   **Dynamic Lists (Model Mappings & Blacklists):**
    *   Visual "Divider" lines separating the main configuration from dynamic lists.
    *   Map items are encased in a light grey box (`#f8fafc`) with a clear visual arrow (`→`) denoting the source-to-target relationship.
    *   Delete actions use a circular hover icon to prevent accidental clicks.
    *   Empty states use a dashed border and centered muted text.

## 5. Success Criteria
*   The Providers page completely eliminates generic "admin panel" components.
*   User experience for managing API keys and mappings feels fluid, modern, and distinctively "premium."
*   Status feedback (blocked vs. active vs. disabled) is immediately recognizable through color hierarchy and opacity without relying solely on reading text.
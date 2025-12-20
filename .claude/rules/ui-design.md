---
paths: "web/style/**/*.css, src/components/**/*.rs"
---

# UI/UX Design Patterns

This rule provides design guidelines for UI development in Arto.

## Design Principles

### Keep it Subtle (控えめに)

- Avoid competing with the main content area
- Use `transparent` backgrounds where possible
- Use thin borders (`1px`) instead of thick (`2px`)
- Prefer `font-weight: 400-500` over bold for navigation
- Icon opacity: `0.5` default, `0.8` on hover/active

### Visual Consistency

- Selected items: `border-color: var(--accent-bg)` + light accent background (`8-10%` opacity)
- All similar buttons must have matching sizes (padding, font-size, border-radius)
- Use `color-mix(in srgb, var(--accent-bg) 8%, transparent)` for subtle selection backgrounds

## In-Page Settings (Browser-Style)

**Prefer in-page settings over modal dialogs for preferences.**

Settings should integrate with the tab system rather than blocking the UI with modals.
This follows browser conventions (Chrome's `chrome://settings`, Firefox's `about:preferences`).

### Architecture

- Add a `TabContent::Preferences` variant to the content enum
- Implement `open_preferences()` method with tab deduplication (reuse existing preferences tab)
- Use state-based navigation instead of broadcast channels for window-specific features

### Layout Structure

```
preferences-page (全体: flex column, min-width: 600px)
│
└─ preferences-page-body (flex row, 両方スクロール可能)
   │
   ├─ preferences-nav (左: width: 180px, 縦並びボタン)
   │  ├─ Theme
   │  ├─ Sidebar
   │  ├─ Directory
   │  ├─ (spacer)
   │  └─ About
   │
   └─ preferences-settings (右: flex: 1)
      ├─ preferences-settings-header
      │  └─ save-status (右寄せ: [Save Changes] or Saving... or Saved!)
      │
      └─ preferences-pane (選択されたタブのコンテンツ)
         ├─ preference-section-title (h3, uppercase)
         └─ preference-item (各設定項目)
            ├─ preference-item-header (label + description)
            └─ Controls (option-cards, theme-selector, slider, etc.)
```

### Key CSS Properties

- Page: `min-width: 600px; overflow-x: auto` (allow horizontal scroll below minimum)
- Navigation: `width: 180px; background: transparent` (don't compete with sidebar)
- Settings header: `min-height: 36px` (prevent layout shift when Save button appears/disappears)

## Form Controls

### Custom Radio Button Styles

**1. Option Cards** - For binary/multiple choices with descriptions:
- Hide native `<input type="radio">` with `opacity: 0; position: absolute`
- Style the `<label>` as a card with `border: 1px solid var(--border-color)`
- Selected: `border-color: var(--accent-bg)` + accent-tinted background
- Separate cards with `gap: 12px` (not connected)

**2. Theme/Icon Selector** - For icon-based choices:
- Same card style as Option Cards (separated, not segmented)
- Icon + label vertically stacked with `gap: 6px`

### Directory/Path Inputs

- Make text input editable (not readonly) for direct path entry
- Use icon button for browse (`FolderOpen` icon, 40x40px square)
- Include "Use Current" button to grab value from current app state

### Slider Inputs

- Combine with value display (`{value}px`) and "Use Current" button
- Keep all related controls on the same row with `gap: 16px`

## Button Sizing Consistency

**All buttons in the same context must match:**

| Button Type | Padding | Font Size | Border Radius |
|-------------|---------|-----------|---------------|
| Primary action (Save) | 8px 16px | 13px | 6px |
| Secondary (Browse, Use Current) | 10px 18px | 14px | 8px |
| Icon button | 0 (40x40px) | - | 8px |

## Typography & Spacing

### Recommended Sizes for Settings Pages

| Element | Font Size | Font Weight |
|---------|-----------|-------------|
| Navigation tab | 13px | 400 (500 when active) |
| Section title | 12px | 600, uppercase |
| Item label | 15px | 600 |
| Description | 13px | 400 |
| Button/Input | 14px | 500 |

### Spacing Guidelines

- Page padding: 24-32px
- Navigation padding: 24px 12px
- Settings content padding: 24px 48px
- Item padding: 20px vertical
- Gap between elements: 12-16px
- Border radius: 8px for cards/inputs, 4-6px for small elements

## Full-Page Content Sections (About, Welcome)

For pages like About, Welcome that fill the entire content area, follow the pattern in `no-file.css`:

```css
.page-container {
  display: flex;
  flex-direction: column;
  align-items: center;      /* Horizontal center */
  /* No justify-content: center - content starts from top */
  width: 100%;
  height: 100%;
  padding: 4rem 2rem;
  text-align: center;
  box-sizing: border-box;
}

.page-content {
  max-width: 500px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  animation: fadeInUp 0.6s ease-out;
}
```

### Key Patterns

- Use `fadeInUp` animation for smooth entry
- Cards for links/hints: `background: var(--bg-secondary); border: 1px solid var(--border-color); border-radius: 0.75rem; padding: 1.5rem`
- Opacity-based text hierarchy: title `0.9`, description `0.6`, footer `0.4`
- Link items: `display: flex; gap: 0.75rem; opacity: 0.7` with hover → `opacity: 1`

### Disable Parent Scroll

When content should fit without scrolling:

```css
.parent-container:has(.full-page) {
  overflow: hidden;
  padding: 0;
}
```

## Menu Integration with Tab Content

**Opening a specific tab when menu item is clicked:**

1. Create a static function to set the tab state before opening:
   ```rust
   // In preferences_view.rs
   static LAST_TAB: LazyLock<Mutex<Tab>> = LazyLock::new(|| Mutex::new(Tab::default()));

   pub fn set_tab_to_about() {
       *LAST_TAB.lock().unwrap() = Tab::About;
   }
   ```

2. Re-export from parent module:
   ```rust
   // In content.rs
   pub use preferences_view::set_tab_to_about;
   ```

3. Call before opening preferences in menu handler:
   ```rust
   // In menu.rs
   MenuId::About => {
       set_preferences_tab_to_about();
       state.open_preferences();
   }
   ```

**Note:** Replace predefined menu items (`PredefinedMenuItem::about`) with custom ones to control navigation.

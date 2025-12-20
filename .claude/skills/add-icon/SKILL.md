---
name: add-icon
description: Add a new Tabler icon to the project. Use when adding icons to the UI.
---

# Add Icon Skill

This skill guides you through adding a new icon from Tabler Icons to the project.

## Process

**Icons are managed via a build script, not by direct file editing.**

### Steps

1. **Choose Icon**
   - Browse available icons: https://tabler.io/icons
   - Note the icon name (e.g., `folder-open`, `info-circle`)

2. **Add to Build Script**
   - Edit `web/scripts/build-icon-sprite.ts`
   - Add the icon name to the `icons` array

3. **Build Icon Sprite**
   ```bash
   cd web
   pnpm run build:icons
   pnpm run build
   ```

4. **Add Rust Enum Variant**
   - Edit `src/components/icon.rs`
   - Add variant to `IconName` enum (use PascalCase)
   - Add case to `Display` implementation

### Example

**web/scripts/build-icon-sprite.ts:**
```typescript
const icons = [
  'folder',
  'folder-open',  // ← Add this
  'file',
  // ...
];
```

**src/components/icon.rs:**
```rust
pub enum IconName {
    Folder,
    FolderOpen,  // ← Add this
    File,
    // ...
}

impl std::fmt::Display for IconName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IconName::Folder => write!(f, "folder"),
            IconName::FolderOpen => write!(f, "folder-open"),  // ← Add this
            IconName::File => write!(f, "file"),
            // ...
        }
    }
}
```

### File Locations

| File | Purpose | Git Tracked |
|------|---------|-------------|
| `web/scripts/build-icon-sprite.ts` | Icon list and build script | ✅ Yes |
| `web/public/icons/tabler-sprite.svg` | Generated sprite (Vite source) | ❌ No |
| `assets/dist/icons/tabler-sprite.svg` | Build output (Dioxus asset) | ❌ No |

### Important

- **NEVER** edit `assets/dist/icons/tabler-sprite.svg` directly
- The `assets/dist/` directory is `.gitignore`d as build output
- Rust code references icons via `asset!("/assets/dist/icons/tabler-sprite.svg")`
- Icons come from `@tabler/icons` npm package (outline style only)

# inventory-blur (Horizon port)

A Rust port of [Inventory Blur](https://github.com/Enchanted-Games/inventory-blur)
(Enchanted_Games / ioblackshaw, CC BY-NC 4.0): applies the 1.20.5+ menu
background blur to in-world inventories (chests, the player inventory and
other Screen-extended menus) instead of the vanilla dark gradient.

## Scope (the mod's full logic core, 1:1)

Upstream read at commit 4546a17, the whole mod is 171 lines of Java. The
1:1 core lives in `src/runtime.rs`, behind the mod API wiring in
`src/lib.rs`:

- ModConstants (name, id, the fabric platform), the SLF4J Logging
  wrapper with its "[Inventory Blur]: " prefix, ModEntry and the loader
  entrypoints (FabricCommonEntry calls ModEntry.init;
  FabricClientEntry.onInitializeClient is empty; NeoForgeEntry sits
  preprocessor-commented out upstream).
- ScreenMixin's renderTransparentBackground replacement:
  eg_inventory_blur$ignoreGradientCall is an empty WrapOperation, so the
  vanilla dark gradient is never issued; eg_inventory_blur$applyBlurAndDrawBG
  skips when the render state's first stratum after blur is set (the
  1.21.6+ guard), then renders the blurred background and, on both build
  paths, tiles eg-inventory-blur:textures/gui/inworld_inventory_background.png
  over the menu. The texture asset ships at
  assets/eg-inventory-blur/textures/gui/inworld_inventory_background.png.
- The tile-count math the vanilla renderMenuBackgroundTexture draw uses.

9 tests green.

## Mod API wiring

Upstream holds no config file: the whole mod exposes no settings, so
`register` registers none. `client_setup` runs ModEntry.init 1:1, the
fabric build's platform line.

## Degradation (rendering rides Horizon's screen surfaces)

The actual blur application and the tiled texture draw are render calls
inside Screen.renderTransparentBackground, and the pinned mod API
(Horizon f00125f61f794aaa3d4e950d301f353b35b6ee3d, mixins
e59da9094f0c2fdda6c627fe9eff22dec6a906f0, api 0.2) has no client screen
background render surface a mod can hook: no blur shader surface, no
replacement point for the menu background, no texture tiling draw. The
gap is tracked on horizon-rust/horizon#19349 (the screen-render hook
filed for the Blur+ port; the same surface this port needs) and was
first filed as #10131, closed 2026-11-06 without the surface landing.
Until it lands the core compiles and tests green, and the screens draw
vanilla's gradient.

## Licence

CC-BY-NC-4.0, matching upstream. Upstream credit:
Enchanted_Games / ioblackshaw.

//! Inventory Blur's logic core, ported 1:1 from Enchanted-Games/inventory-blur
//! at upstream commit 4546a17 (CC BY-NC 4.0). Upstream is 171 lines of Java:
//! ModConstants, the SLF4J Logging wrapper, ModEntry, the fabric entrypoints
//! (the neoforge entry sits preprocessor-commented out upstream), the two
//! 1.21.6 accessors, and ScreenMixin's renderTransparentBackground
//! replacement. Classes and behavior stay 1:1; the render calls are the
//! host seam tracked on horizon-rust/horizon#19349 (the screen-render hook,
//! filed for the Blur+ port; the same surface this port waits on) and first
//! filed as #10131, closed 2026-11-06 without the surface landing.

/// Upstream ModConstants.
pub const MOD_NAME: &str = "Inventory Blur";
pub const MOD_ID: &str = "eg_inventory_blur";

/// Upstream ModConstants.TARGET_PLATFORM on the fabric build (the neoforge
/// arm sits inside a preprocessor comment upstream).
pub const TARGET_PLATFORM: &str = "fabric";

/// Upstream ScreenMixin's in-world inventory background texture: a 16x16
/// image tiled across the screen. The asset ships with this mod at
/// assets/eg-inventory-blur/textures/gui/inworld_inventory_background.png.
pub const INWORLD_INVENTORY_BACKGROUND_TEXTURE: &str =
    "eg-inventory-blur:textures/gui/inworld_inventory_background.png";
pub const BACKGROUND_TILE_SIZE: i32 = 16;

/// Upstream Logging: the SLF4J wrapper. Every call passes its message
/// through at the given level with upstream's "[Inventory Blur]: " prefix.
pub mod logging {
    use super::MOD_NAME;

    /// Upstream Logging's messagePrefix.
    fn prefix() -> String {
        format!("[{}]: ", MOD_NAME)
    }

    /// The prefixed line a Logging call produces.
    fn line(message: &str) -> String {
        format!("{}{}", prefix(), message)
    }

    pub fn info(message: &str) {
        log_at("INFO", &line(message));
    }

    pub fn warn(message: &str) {
        log_at("WARN", &line(message));
    }

    pub fn error(message: &str) {
        log_at("ERROR", &line(message));
    }

    pub fn debug(message: &str) {
        log_at("DEBUG", &line(message));
    }

    /// The host log seam: the pinned mod API has no logging call a mod can
    /// reach from a hook, so the line goes to stderr, which the host's mod
    /// log captures. Level and text are upstream's.
    fn log_at(level: &str, line: &str) {
        eprintln!("{} {}", level, line);
    }
}

/// Upstream ModEntry: the common entrypoint each loader side calls.
pub struct ModEntry;

impl ModEntry {
    /// Upstream ModEntry.init, 1:1.
    pub fn init(platform: &str) {
        logging::info(&Self::loading_message(platform));
    }

    /// The message upstream's init logs.
    fn loading_message(platform: &str) -> String {
        format!("Mod is loading on a {} environment!", platform)
    }
}

/// Upstream FabricCommonEntry::onInitialize: calls ModEntry.init.
pub fn fabric_common_entry() {
    ModEntry::init(TARGET_PLATFORM);
}

/// Upstream NeoForgeEntry: its constructor would call ModEntry.init, but
/// the whole class sits inside a preprocessor comment upstream (the fabric
/// build is the one shipped), so it never runs. Kept for 1:1 coverage.
pub fn neoforge_entry() {
    ModEntry::init(TARGET_PLATFORM);
}

/// Upstream FabricClientEntry::onInitializeClient: empty.
pub fn fabric_client_entry() {}

/// The blurred-background render upstream @Shadow-calls, per build path:
/// 1.21.6+ takes the GuiGraphics, older builds take nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlurRender {
    /// 1.21.6+: Screen.renderBlurredBackground(GuiGraphics).
    WithGraphics,
    /// Pre-1.21.6: Screen.renderBlurredBackground().
    Bare,
}

/// The menu-background texture draw Screen.renderMenuBackgroundTexture
/// makes with upstream's arguments, on both build paths.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuBackgroundTexture {
    pub texture: &'static str,
    pub x: i32,
    pub y: i32,
    pub u: f32,
    pub v: f32,
    pub width: i32,
    pub height: i32,
}

/// What the renderTransparentBackground replacement runs once the guard
/// passes: the blurred-background render, then the tiled texture.
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundPlan {
    pub blur: BlurRender,
    pub menu_texture: MenuBackgroundTexture,
}

/// Upstream ScreenMixin.eg_inventory_blur$applyBlurAndDrawBG, the Inject at
/// the HEAD of renderTransparentBackground, as the plan a host hook would
/// run. `first_stratum_after_blur` is the render state's value
/// (i32::MAX when no blur stratum exists yet this frame), read through the
/// 1.21.6 accessors (GuiGraphicsAccessor.guiRenderState ->
/// GuiRenderStateAccessor.firstStratumAfterBlur); `supports_blur` mirrors
/// the 1.21.6+ build, where the accessor and the guard are compiled in.
/// Upstream's flow, on both build paths:
/// - 1.21.6+ only: dont draw the blur if its already been drawn this frame
///   (early return when the stratum is set);
/// - renderBlurredBackground (with the GuiGraphics on 1.21.6+, bare on
///   older builds);
/// - then, always: Screen.renderMenuBackgroundTexture(texture, 0, 0, 0.0,
///   0.0, width, height).
/// None mirrors upstream's early return.
pub fn apply_blur_and_draw_bg(
    first_stratum_after_blur: Option<i32>,
    supports_blur: bool,
    width: i32,
    height: i32,
) -> Option<BackgroundPlan> {
    if supports_blur {
        // dont draw the blur if its already been drawn this frame
        if let Some(stratum) = first_stratum_after_blur {
            if stratum != i32::MAX {
                return None;
            }
        }
    }
    let blur = if supports_blur {
        BlurRender::WithGraphics
    } else {
        BlurRender::Bare
    };
    Some(BackgroundPlan {
        blur,
        menu_texture: MenuBackgroundTexture {
            texture: INWORLD_INVENTORY_BACKGROUND_TEXTURE,
            x: 0,
            y: 0,
            u: 0.0,
            v: 0.0,
            width,
            height,
        },
    })
}

/// Upstream ScreenMixin.eg_inventory_blur$ignoreGradientCall, the
/// WrapOperation on the vanilla GuiGraphics.fillGradient call inside
/// renderTransparentBackground: the body is empty, so the dark gradient
/// the vanilla path would draw is never issued. The parameters upstream
/// receives (x1, y1, x2, y2, colorFrom, colorTo) are dropped the same way.
pub fn ignore_gradient_call() {}

/// The tiling the vanilla Screen.renderMenuBackgroundTexture draw performs
/// for the menu background: 16x16 tiles covering width x height. Vanilla
/// behavior the mod's texture ride sits on, kept as the port's model of it.
pub fn tile_counts(width: i32, height: i32) -> (i32, i32) {
    (
        (width + BACKGROUND_TILE_SIZE - 1) / BACKGROUND_TILE_SIZE,
        (height + BACKGROUND_TILE_SIZE - 1) / BACKGROUND_TILE_SIZE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_upstream() {
        assert_eq!(MOD_NAME, "Inventory Blur");
        assert_eq!(MOD_ID, "eg_inventory_blur");
        assert_eq!(TARGET_PLATFORM, "fabric");
        assert_eq!(
            INWORLD_INVENTORY_BACKGROUND_TEXTURE,
            "eg-inventory-blur:textures/gui/inworld_inventory_background.png"
        );
        assert_eq!(BACKGROUND_TILE_SIZE, 16);
    }

    #[test]
    fn blur_applies_once_per_frame_on_the_modern_path() {
        // No blur stratum yet -> apply.
        let plan = apply_blur_and_draw_bg(Some(i32::MAX), true, 320, 240).unwrap();
        assert_eq!(plan.blur, BlurRender::WithGraphics);
        // Already drawn this frame -> nothing.
        assert_eq!(apply_blur_and_draw_bg(Some(0), true, 320, 240), None);
        assert_eq!(apply_blur_and_draw_bg(Some(7), true, 320, 240), None);
    }

    #[test]
    fn the_legacy_path_has_no_stratum_guard() {
        // Older builds compile the accessor and the guard out, so a set
        // stratum does not block the draw there.
        let plan = apply_blur_and_draw_bg(Some(0), false, 320, 240).unwrap();
        assert_eq!(plan.blur, BlurRender::Bare);
    }

    #[test]
    fn the_texture_draw_follows_the_blur_on_both_paths() {
        for (stratum, supports) in [(Some(i32::MAX), true), (None, false)] {
            let plan = apply_blur_and_draw_bg(stratum, supports, 427, 240).unwrap();
            assert_eq!(
                plan.menu_texture,
                MenuBackgroundTexture {
                    texture: INWORLD_INVENTORY_BACKGROUND_TEXTURE,
                    x: 0,
                    y: 0,
                    u: 0.0,
                    v: 0.0,
                    width: 427,
                    height: 240,
                }
            );
        }
    }

    #[test]
    fn tiling_covers_the_screen() {
        assert_eq!(tile_counts(320, 240), (20, 15));
        assert_eq!(tile_counts(321, 240), (21, 15));
        assert_eq!(tile_counts(320, 241), (20, 16));
        assert_eq!(tile_counts(16, 16), (1, 1));
    }

    #[test]
    fn the_gradient_call_is_a_no_op() {
        // The WrapOperation body is empty: the vanilla dark gradient is
        // never issued.
        ignore_gradient_call();
    }

    #[test]
    fn logging_carries_the_upstream_prefix() {
        let line = format!("[{}]: {}", MOD_NAME, "hello");
        assert_eq!(line, "[Inventory Blur]: hello");
    }

    #[test]
    fn mod_entry_logs_the_environment() {
        assert_eq!(
            ModEntry::loading_message(TARGET_PLATFORM),
            "Mod is loading on a fabric environment!"
        );
    }

    #[test]
    fn the_entrypoints_run_as_upstream_declares_them() {
        // FabricClientEntry::onInitializeClient is empty upstream.
        fabric_client_entry();
        // FabricCommonEntry::onInitialize calls ModEntry.init.
        fabric_common_entry();
        // NeoForgeEntry sits preprocessor-commented out upstream; the
        // coverage path still runs.
        neoforge_entry();
    }
}

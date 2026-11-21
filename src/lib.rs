//! Inventory Blur for Horizon: the mod API wiring.
//!
//! The 1:1 logic core lives in `runtime.rs`, ported from
//! Enchanted-Games/inventory-blur at 4546a17 (CC BY-NC 4.0): ModConstants,
//! the SLF4J Logging wrapper, ModEntry, the loader entrypoints and
//! ScreenMixin's renderTransparentBackground replacement (the gradient
//! suppression, the once-per-frame blur guard, the blurred-background
//! render and the menu-background texture draw).
//!
//! Upstream has no config file: the whole mod is 171 lines and holds no
//! settings, so `register` registers none. `client_setup` runs
//! ModEntry.init 1:1, the fabric build's platform line.
//!
//! The draw is a seam: upstream applies the blur and tiles
//! eg-inventory-blur:textures/gui/inworld_inventory_background.png inside
//! Screen.renderTransparentBackground, and the pinned mod API (f00125f61f,
//! mixins e59da909, api 0.2) has no client screen background render
//! surface a mod can hook. The gap is tracked on horizon-rust/horizon#19349
//! (the screen-render hook filed for the Blur+ port; the same surface this
//! port needs) and was first filed as #10131, closed 2026-11-06 without
//! the surface landing. Until it lands the core compiles and tests green,
//! and the screens draw vanilla's gradient.
use mod_api::{self, ClientHost, ClientSetupEvent, RegistrationContext};

mod runtime;

use runtime::{ModEntry, TARGET_PLATFORM};

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// Upstream has no config file and exposes no settings, so nothing
    /// registers on Horizon's Options > Mods page.
    fn register(&self, _registrar: &mut dyn mod_api::Registrar, _context: &RegistrationContext) {
    }

    /// ModEntry.init, 1:1: logs the environment the mod loads on.
    fn client_setup(&self, _host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        ModEntry::init(TARGET_PLATFORM);
    }
}

//! Material Design 3 for Bevy's UI.
//!
//! Every component is drawn over Bevy's own UI: laid out by `bevy_ui`, and
//! behaving as the headless widgets of `bevy_ui_widgets` do - a button is
//! activated by a click, or by Enter or Space while focused; a slider is
//! dragged, clicked or moved with the arrow keys. This crate draws them as
//! Material's, from one theme ([`theme::MaterialTheme`]): colour roles, a
//! type scale, shapes, elevation and state layers.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_material_ui::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((DefaultPlugins, MaterialUiPlugin))
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2d);
//!     commands
//!         .spawn(paper().bundle())
//!         .with_children(|paper| {
//!             paper.spawn(text("Hello", TypeRole::HeadlineSmall, ColorRole::OnSurface));
//!             paper
//!                 .spawn(button("Play").bundle())
//!                 .observe(|_: On<Activate>| info!("Play"));
//!         });
//! }
//! ```
//!
//! A component is a bundle - a function or a builder's `bundle()` - that the
//! game spawns where it likes, with its own components beside it. What a
//! control does is the game's: it observes the control's events
//! ([`bevy::ui_widgets::Activate`], [`bevy::ui_widgets::ValueChange`]) and
//! keeps the state. Nothing here knows what it changes.

mod button;
mod focus;
mod icon;
mod paint;
mod paper;
pub mod testing;
pub mod theme;

use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::input_focus::{InputDispatchPlugin, InputFocusPlugin};
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::ui_widgets::UiWidgetsPlugins;

pub use crate::button::{
    BUTTON_HEIGHT, ButtonVariant, Fab, FabSize, ICON_BUTTON, IconButtonVariant, MaterialButton,
    button, fab, icon_button,
};
pub use crate::focus::FocusRing;
pub use crate::icon::{Glyph, ICON, Icon, IconRoot, icon};
pub use crate::paint::{
    Content, ContentColor, Fill, Ink, Look, Paint, Raised, Stroke, Typeset, label, set_enabled,
    text,
};
pub use crate::paper::{CardKind, Paper, card, divider, paper};

/// Everything a game needs to draw a Material interface.
pub mod prelude {
    pub use bevy::ui_widgets::{Activate, ValueChange};

    pub use crate::theme::{
        Brightness, ColorRole, Corner, Elevation, MaterialFonts, MaterialTheme, Scheme, Space,
        TypeRole,
    };
    pub use crate::*;
}

/// Draws Material's components: the theme, the painting of every part from
/// it, the controls' states and the focus ring. Adds Bevy's headless widgets
/// and keyboard focus too, unless the game has added them already.
pub struct MaterialUiPlugin;

impl Plugin for MaterialUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InputFocusPlugin>() {
            app.add_plugins(InputFocusPlugin);
        }
        if !app.is_plugin_added::<InputDispatchPlugin>() {
            app.add_plugins(InputDispatchPlugin);
        }
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_plugins(TabNavigationPlugin);
        }
        if !app.is_plugin_added::<bevy::ui_widgets::ButtonPlugin>() {
            app.add_plugins(UiWidgetsPlugins);
        }
        app.init_resource::<theme::MaterialTheme>()
            .add_observer(paint::on_fill)
            .add_observer(paint::on_stroke)
            .add_observer(paint::on_ink)
            .add_observer(paint::on_typeset)
            .add_observer(paint::on_raised)
            .add_observer(button::on_starts_enabled)
            .add_systems(
                PostUpdate,
                (
                    paint::repaint_on_theme_change,
                    paint::paint_controls,
                    paint::paint_content,
                    focus::draw_focus_rings,
                )
                    .chain()
                    .before(UiSystems::Prepare),
            );
        #[cfg(feature = "roboto")]
        app.add_systems(PreStartup, theme::add_roboto);
    }
}

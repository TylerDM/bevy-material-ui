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

// The core: painting from the theme, focus, icons.
mod focus;
mod icon;
mod paint;
pub mod testing;
pub mod theme;
// Surfaces and buttons.
mod button;
mod paper;
mod segmented;
// Selection controls.
mod checkbox;
mod radio;
mod slider;
mod switch;
// Text, search and chips.
mod chip;
mod search;
mod text_field;
// Navigation, lists and menus.
mod app_bar;
mod list;
mod menu;
mod navigation;
mod tabs;
// Feedback, overlays and display.
mod badge;
mod dialog;
mod progress;
mod scroll;
mod sheet;
mod skeleton;
mod snackbar;
mod tooltip;
// The play test.
pub mod gallery;

use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::input_focus::{InputDispatchPlugin, InputFocusPlugin};
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::ui_widgets::UiWidgetsPlugins;

// The core.
pub use crate::focus::FocusRing;
pub use crate::icon::{Glyph, ICON, Icon, IconRoot, icon};
pub use crate::paint::{
    Content, ContentColor, Fill, Ink, Look, Paint, Raised, Stroke, Typeset, label, set_enabled,
    text,
};
// Surfaces and buttons.
pub use crate::button::{
    BUTTON_HEIGHT, ButtonVariant, Fab, FabSize, ICON_BUTTON, IconButtonVariant, MaterialButton,
    button, fab, icon_button,
};
pub use crate::paper::{CardKind, Paper, card, divider, paper};
pub use crate::segmented::{
    MaterialSegmentedButtons, SEGMENT_HEIGHT, Segment, SegmentCheck, SegmentStyle,
    SegmentedButtons, chosen_after, segment_look, segmented_buttons,
};
// Selection controls.
pub use crate::checkbox::{
    BOX, CheckboxBox, CheckboxMark, CheckboxPaints, CheckboxStyle, LabelRow, MaterialCheckbox,
    SelfUpdating, TARGET, checkbox, checkbox_paints,
};
pub use crate::radio::{
    DOT, MaterialRadio, MaterialRadioGroup, RING, RadioDot, RadioOption, RadioRing, RadioStyle,
    radio, radio_group, radio_paints,
};
pub use crate::slider::{
    HANDLE, KEY_SHARE, MaterialSlider, MaterialSliderStyle, NotchLabel, Notches, SLIDER_HEIGHT,
    SLIDER_TRACK, SliderFill, SliderHalo, SliderStop, SliderTrack, key_step, notch_at,
    notched_slider, on_a_step, set_notch, slider,
};
pub use crate::switch::{
    MaterialSwitch, SWITCH_TRACK, SwitchHalo, SwitchHandle, SwitchPaints, SwitchState, SwitchStyle,
    halo_left, handle_side, switch, switch_paints,
};
// Text, search and chips.
pub use crate::chip::{
    CHIP_HEIGHT, ChipKind, ChipRemoved, MaterialChip, chip, chip_border, chip_look, chip_set,
};
pub use crate::search::{
    SEARCH_HEIGHT, SearchBar, SearchNavigate, SearchResult, search_bar, search_result,
    search_results,
};
pub use crate::text_field::{
    FIELD_HEIGHT, InputText, MaterialTextField, TextField, TextFieldVariant, counter_text,
    set_input_text, text_field,
};
// Navigation, lists and menus.
pub use crate::app_bar::{
    AppBar, AppBarAction, AppBarButton, AppBarKind, AppBarTitle, TopAppBar, top_app_bar,
};
pub use crate::list::{
    Leading, Lines, ListRow, MaterialList, MaterialListItem, Trailing, TrailingSlot, list,
    list_item, list_unmanaged, row_look,
};
pub use crate::menu::{
    MaterialMenu, MaterialMenuRoot, MaterialSelect, MenuChoice, MenuEntry, MenuLine, MenuOpener,
    MenuSurface, MenuTrigger, SelectField, SelectValue, item_look, menu, menu_item, select,
};
pub use crate::navigation::{
    ActiveIndicator, Destination, Navigation, NavigationDestination, NavigationDrawer,
    NavigationKind, NavigationSelection, RailFab, RailFabActivated, destination, indicator_look,
    label_role, navigation_bar, navigation_drawer, navigation_rail,
};
pub use crate::tabs::{
    MaterialTab, MaterialTabHeader, MaterialTabs, MaterialTabsRow, TabIndicator, TabPanel,
    TabPanels, TabsKind, tab, tab_panel, tab_panels, tabs,
};
// Feedback, overlays and display.
pub use crate::badge::{
    AVATAR, Avatar, AvatarFace, BADGE_MAX, Badge, BadgeLabel, LARGE_BADGE, SMALL_BADGE, avatar,
    badge, badge_label, initials,
};
pub use crate::dialog::{
    CANCEL, CONFIRM, DialogAction, DialogActionKey, DialogAnswer, DialogResult, DialogScrim,
    DialogSurface, MaterialDialog, SCRIM_OPACITY, confirm_dialog, dialog,
};
pub use crate::progress::{
    CIRCULAR_SIDE, CIRCULAR_STROKE, CircularProgress, LINEAR_THICKNESS, LinearProgress, Progress,
    ProgressClock, ProgressIndicator, SWEEP_SECONDS, arc_gradient, circular_arc, circular_progress,
    linear_progress, linear_span,
};
pub use crate::scroll::{
    MaterialScrollThumb, MaterialScrollbar, SCROLLBAR_GUTTER, ScrollViewport, THUMB_REST_OPACITY,
    overflows, scroll_area,
};
pub use crate::sheet::{
    SHEET_MAX_WIDTH, SheetClosed, SheetHandle, SheetScrim, SheetSurface, bottom_sheet,
};
pub use crate::skeleton::{PULSE_SECONDS, Skeleton, pulse, skeleton};
pub use crate::snackbar::{
    SNACKBAR_ACTION_SECONDS, SNACKBAR_SECONDS, Snackbar, SnackbarAction, SnackbarActionButton,
    SnackbarCloseButton, SnackbarCloseReason, SnackbarClosed, SnackbarId, SnackbarView, Snackbars,
};
pub use crate::tooltip::{
    TOOLTIP_DELAY, Tooltip, TooltipAction, TooltipKind, TooltipState, TooltipView,
};

/// Everything a game needs to draw a Material interface.
pub mod prelude {
    pub use bevy::ui_widgets::{Activate, ValueChange};

    pub use crate::theme::{
        Brightness, ColorRole, Corner, Elevation, MaterialFonts, MaterialTheme, Scheme, Space,
        TypeRole, hex,
    };
    pub use crate::*;
}

/// The systems that paint every part from the theme, in `PostUpdate`: a
/// component's own systems that change a [`Look`] or a [`ContentColor`] run
/// before them, so the change is drawn the same frame.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PaintSystems;

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
                    .in_set(PaintSystems),
            )
            .configure_sets(PostUpdate, PaintSystems.before(UiSystems::Prepare));
        segmented::plugin(app);
        checkbox::plugin(app);
        radio::plugin(app);
        slider::plugin(app);
        switch::plugin(app);
        chip::plugin(app);
        search::plugin(app);
        text_field::plugin(app);
        app_bar::plugin(app);
        list::plugin(app);
        menu::plugin(app);
        navigation::plugin(app);
        tabs::plugin(app);
        badge::plugin(app);
        dialog::plugin(app);
        progress::plugin(app);
        scroll::plugin(app);
        sheet::plugin(app);
        skeleton::plugin(app);
        snackbar::plugin(app);
        tooltip::plugin(app);
        #[cfg(feature = "roboto")]
        app.add_systems(PreStartup, theme::add_roboto);
    }
}

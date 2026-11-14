//! Tooltips: a few words about a part, shown when the pointer rests on it.
//!
//! Put a [`Tooltip`] on any part. After the pointer has rested on it for
//! [`TOOLTIP_DELAY`] the tooltip appears beside it - above where there is
//! room, below where there isn't - and goes when the pointer leaves. A rich
//! tooltip can carry a title and an action, which sends [`TooltipAction`];
//! the pointer can move onto it without it going.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

use crate::button::button;
use crate::paint::{ContentColor, Fill, Paint, Raised, text};
use crate::theme::{ColorRole, Corner, Elevation, Space, TypeRole};

/// How long the pointer rests on a part before its tooltip appears, in
/// seconds.
pub const TOOLTIP_DELAY: f32 = 0.5;

/// The layer tooltips are drawn in: above the game and its snackbars.
const TOOLTIP_LAYER: i32 = 100;

/// Plain or rich.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum TooltipKind {
    /// A short label on the inverse surface.
    #[default]
    Plain,
    /// A card of words, with an optional title and action.
    Rich,
}

/// A tooltip for the part it is on.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
#[require(Hovered, TooltipState)]
#[reflect(Component)]
pub struct Tooltip {
    /// What it says.
    pub text: String,
    /// Plain or rich.
    pub kind: TooltipKind,
    /// A rich tooltip's title.
    pub title: Option<String>,
    /// A rich tooltip's action's label.
    pub action: Option<String>,
}

impl Tooltip {
    /// A plain tooltip saying `text`.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    /// A rich tooltip saying `text`.
    pub fn rich(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: TooltipKind::Rich,
            ..default()
        }
    }

    /// With a title, which makes it rich.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.kind = TooltipKind::Rich;
        self.title = Some(title.into());
        self
    }

    /// With an action saying `label`, which makes it rich.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.kind = TooltipKind::Rich;
        self.action = Some(label.into());
        self
    }
}

/// How long the pointer has rested on a part, and its tooltip if shown.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct TooltipState {
    /// Seconds the pointer has rested on it.
    pub rested: f32,
    /// Its tooltip, while shown.
    pub shown: Option<Entity>,
}

/// Marks a tooltip on screen.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct TooltipView {
    /// The part it is about.
    pub anchor: Entity,
}

/// Marks a rich tooltip's action button.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
struct TooltipActionButton(Entity);

/// Sent to the part a rich tooltip is about when its action is used.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipAction {
    /// The part with the tooltip.
    #[event_target]
    pub anchor: Entity,
}

/// Where a tooltip goes beside its part: above, centred, or else below.
fn placement() -> Popover {
    let at = |side| PopoverPlacement {
        side,
        align: PopoverAlign::Center,
        gap: Space::ExtraSmall.px(),
    };
    Popover {
        positions: vec![at(PopoverSide::Top), at(PopoverSide::Bottom)],
        window_margin: Space::Small.px(),
    }
}

/// A tooltip's parts, about `anchor`.
fn tooltip_bundle(anchor: Entity, tooltip: &Tooltip) -> impl Bundle {
    let rich = tooltip.kind == TooltipKind::Rich;
    let words = tooltip.text.clone();
    let title = tooltip.title.clone();
    let action = tooltip.action.clone();
    let (surface, content, corner, padding) = if rich {
        (
            ColorRole::SurfaceContainer,
            ColorRole::OnSurfaceVariant,
            Corner::Medium,
            UiRect::new(
                Space::Large.val(),
                Space::Large.val(),
                Space::Medium.val(),
                Space::Small.val(),
            ),
        )
    } else {
        (
            ColorRole::InverseSurface,
            ColorRole::InverseOnSurface,
            Corner::ExtraSmall,
            UiRect::axes(Space::Small.val(), Space::ExtraSmall.val()),
        )
    };
    (
        TooltipView { anchor },
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            max_width: Val::Px(if rich { 320.0 } else { 200.0 }),
            min_height: Val::Px(if rich { 0.0 } else { 24.0 }),
            justify_content: JustifyContent::Center,
            padding,
            row_gap: Space::ExtraSmall.val(),
            border_radius: corner.all(),
            ..default()
        },
        placement(),
        GlobalZIndex(TOOLTIP_LAYER),
        Fill::new(surface),
        ContentColor(Paint::new(content)),
        Raised(if rich {
            Elevation::Level2
        } else {
            Elevation::Level0
        }),
        Children::spawn(SpawnWith(move |inside: &mut ChildSpawner| {
            if let Some(title) = title {
                inside.spawn(text(
                    title,
                    TypeRole::TitleSmall,
                    ColorRole::OnSurfaceVariant,
                ));
            }
            let kind = if rich {
                TypeRole::BodyMedium
            } else {
                TypeRole::BodySmall
            };
            inside.spawn(text(words, kind, content));
            if let Some(label) = action {
                inside
                    .spawn(Node {
                        padding: UiRect::top(Space::Small.val()),
                        margin: UiRect::left(Val::Px(-12.0)),
                        ..default()
                    })
                    .with_child((button(label).text().bundle(), TooltipActionButton(anchor)));
            }
        })),
    )
}

/// Shows each part's tooltip once the pointer has rested on it, and takes
/// it away when the pointer leaves.
fn run_tooltips(
    time: Res<Time>,
    mut anchors: Query<(Entity, &Tooltip, &Hovered, &mut TooltipState)>,
    mut commands: Commands,
) {
    for (anchor, tooltip, hovered, mut state) in &mut anchors {
        if !hovered.0 {
            if let Some(shown) = state.shown.take() {
                commands.entity(shown).try_despawn();
            }
            if state.rested != 0.0 {
                state.rested = 0.0;
            }
            continue;
        }
        state.rested += time.delta_secs();
        if state.shown.is_none() && state.rested >= TOOLTIP_DELAY {
            let mut view = commands.spawn((tooltip_bundle(anchor, tooltip), ChildOf(anchor)));
            if tooltip.kind == TooltipKind::Plain {
                view.insert(Pickable::IGNORE);
            }
            state.shown = Some(view.id());
        }
    }
}

/// Sends a rich tooltip's action to its part.
fn on_activate(
    activate: On<Activate>,
    buttons: Query<&TooltipActionButton>,
    mut commands: Commands,
) {
    if let Ok(TooltipActionButton(anchor)) = buttons.get(activate.entity) {
        commands.trigger(TooltipAction { anchor: *anchor });
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_activate)
        .add_systems(Update, run_tooltips);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_or_an_action_makes_a_tooltip_rich() {
        assert_eq!(Tooltip::plain("Save").kind, TooltipKind::Plain);
        assert_eq!(
            Tooltip::plain("Save").title("Saving").kind,
            TooltipKind::Rich
        );
        assert_eq!(
            Tooltip::plain("Save").action("Learn more").kind,
            TooltipKind::Rich
        );
    }

    #[test]
    fn a_tooltip_goes_above_its_part_where_there_is_room_and_below_where_not() {
        let sides: Vec<PopoverSide> = placement().positions.iter().map(|at| at.side).collect();
        assert_eq!(sides, [PopoverSide::Top, PopoverSide::Bottom]);
    }
}

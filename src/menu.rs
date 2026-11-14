//! Menus, and the select field that opens one.
//!
//! A [`menu`] is a button and a popup of items over Bevy's headless menus
//! ([`MenuButton`], [`MenuPopup`], [`MenuItem`]): the button, or an arrow key
//! while it has the keyboard's focus, opens the popup beside it; the arrow
//! keys, Home and End move between items, and a click or Enter or Space
//! chooses one; Escape, or a click anywhere else, closes it. Choosing an item
//! sends [`ValueChange<usize>`] - the item's index, dividers not counted - on
//! the menu.
//!
//! A [`select`] is a field showing the chosen option, opening a menu of the
//! options; choosing one shows it, and sends the same [`ValueChange<usize>`]
//! on the field.

use bevy::input_focus::tab_navigation::{NavAction, TabIndex};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui::OverrideClip;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};
use bevy::ui_widgets::{
    Activate, MenuAction, MenuButton, MenuEvent, MenuFocusState, MenuItem, MenuPopup, ValueChange,
    observe,
};

use crate::button::{IconButtonVariant, MaterialButton, icon_button};
use crate::focus::FocusRing;
use crate::icon::{Glyph, Icon};
use crate::list::{quiet_slot, quiet_text};
use crate::paint::{ContentColor, Fill, Look, Paint, Raised, Stroke, label};
use crate::theme::{ColorRole, Corner, Elevation, TypeRole};

/// An item of a menu: its words, icons, shortcut and state.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuEntry {
    /// What it says.
    pub label: String,
    /// An icon before its words.
    pub leading: Option<Icon>,
    /// An icon after its words.
    pub trailing: Option<Icon>,
    /// The keys that do the same, shown after its words: "Ctrl+C".
    pub shortcut: Option<String>,
    /// Whether it can be chosen.
    pub enabled: bool,
    /// Whether it is the one chosen, drawn in the secondary container.
    pub selected: bool,
}

/// An item saying `label`.
pub fn menu_item(label: impl Into<String>) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        leading: None,
        trailing: None,
        shortcut: None,
        enabled: true,
        selected: false,
    }
}

impl MenuEntry {
    /// With `icon` before its words.
    pub fn leading(mut self, icon: impl Into<Icon>) -> Self {
        self.leading = Some(icon.into());
        self
    }

    /// With `icon` after its words.
    pub fn trailing(mut self, icon: impl Into<Icon>) -> Self {
        self.trailing = Some(icon.into());
        self
    }

    /// With `keys` shown after its words.
    pub fn shortcut(mut self, keys: impl Into<String>) -> Self {
        self.shortcut = Some(keys.into());
        self
    }

    /// Enabled, or drawn faded and unchoosable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Drawn as the one chosen, or not.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// A line of a menu: an item, or a divider between groups of them.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuLine {
    /// An item.
    Item(MenuEntry),
    /// A rule between groups of items.
    Divider,
}

impl From<MenuEntry> for MenuLine {
    fn from(entry: MenuEntry) -> Self {
        MenuLine::Item(entry)
    }
}

/// What opens a menu.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuTrigger {
    /// A button with words.
    Button(MaterialButton),
    /// An icon button: three dots, say.
    IconButton(Icon, IconButtonVariant),
}

impl From<MaterialButton> for MenuTrigger {
    fn from(button: MaterialButton) -> Self {
        MenuTrigger::Button(button)
    }
}

/// Marks a menu: the button and the popup are inside it, and it sends
/// [`ValueChange<usize>`] when an item is chosen.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialMenuRoot;

/// Marks what opens a menu.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MenuOpener;

/// Marks a menu's popup.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MenuSurface;

/// An item's index among its menu's items.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[component(immutable)]
#[reflect(Component)]
pub struct MenuChoice(pub usize);

/// A menu: what opens it and its lines.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialMenu {
    /// What opens it.
    pub trigger: MenuTrigger,
    /// Its items and dividers, in order.
    pub lines: Vec<MenuLine>,
}

/// A menu opened by `trigger`, with no items yet: add them with the builder.
pub fn menu(trigger: impl Into<MenuTrigger>) -> MaterialMenu {
    MaterialMenu {
        trigger: trigger.into(),
        lines: Vec::new(),
    }
}

impl MaterialMenu {
    /// With `entry` after the lines already added.
    pub fn item(mut self, entry: MenuEntry) -> Self {
        self.lines.push(MenuLine::Item(entry));
        self
    }

    /// With a divider after the lines already added.
    pub fn divider(mut self) -> Self {
        self.lines.push(MenuLine::Divider);
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let MaterialMenu { trigger, lines } = self;
        (
            MaterialMenuRoot,
            observe(on_menu_event),
            Node::default(),
            Children::spawn(SpawnWith(move |root: &mut ChildSpawner| {
                match trigger {
                    MenuTrigger::Button(button) => {
                        root.spawn((button.bundle(), MenuButton, MenuOpener));
                    }
                    MenuTrigger::IconButton(icon, variant) => {
                        root.spawn((icon_button(icon, variant), MenuButton, MenuOpener));
                    }
                }
                root.spawn(popup(Val::Px(112.0)))
                    .with_children(|popup| spawn_lines(popup, lines));
            })),
        )
    }
}

/// A menu's popup, at least `min_width` wide, hidden until opened.
fn popup(min_width: Val) -> impl Bundle {
    (
        (
            MenuSurface,
            MenuPopup::default(),
            Popover {
                positions: vec![
                    PopoverPlacement {
                        side: PopoverSide::Bottom,
                        align: PopoverAlign::Start,
                        gap: 4.0,
                    },
                    PopoverPlacement {
                        side: PopoverSide::Top,
                        align: PopoverAlign::Start,
                        gap: 4.0,
                    },
                ],
                window_margin: 8.0,
            },
            Visibility::Hidden,
            GlobalZIndex(100),
            OverrideClip,
        ),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            min_width,
            max_width: Val::Px(280.0),
            padding: UiRect::vertical(Val::Px(8.0)),
            border_radius: Corner::ExtraSmall.all(),
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainer),
        Raised(Elevation::Level2),
        ContentColor(Paint::new(ColorRole::OnSurface)),
    )
}

/// Spawns a menu's `lines` into its popup.
fn spawn_lines(popup: &mut ChildSpawner, lines: Vec<MenuLine>) {
    let mut index = 0;
    for line in lines {
        match line {
            MenuLine::Item(entry) => {
                popup.spawn(entry_bundle(index, entry));
                index += 1;
            }
            MenuLine::Divider => {
                popup
                    .spawn((
                        Node {
                            padding: UiRect::vertical(Val::Px(8.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_child(crate::divider(false));
            }
        }
    }
}

/// The look of a menu item, chosen or not.
pub fn item_look(selected: bool) -> Look {
    if selected {
        Look::filled(
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
        )
    } else {
        Look::bare(ColorRole::OnSurface)
    }
}

/// An item of a menu, at `index` among its items.
fn entry_bundle(index: usize, entry: MenuEntry) -> impl Bundle {
    let MenuEntry {
        label: words,
        leading,
        trailing,
        shortcut,
        enabled,
        selected,
    } = entry;
    (
        (
            MenuItem,
            MenuChoice(index),
            item_look(selected),
            TabIndex(0),
            FocusRing,
            crate::button::enabled(enabled),
        ),
        Node {
            height: Val::Px(48.0),
            padding: UiRect::horizontal(Val::Px(12.0)),
            column_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |item: &mut ChildSpawner| {
            if let Some(icon) = leading {
                item.spawn(quiet_slot(ColorRole::OnSurfaceVariant))
                    .with_child(crate::icon::icon(icon, crate::icon::ICON));
            }
            item.spawn((
                label(words, TypeRole::BodyLarge),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            if let Some(keys) = shortcut {
                item.spawn(quiet_text(
                    keys,
                    TypeRole::LabelLarge,
                    ColorRole::OnSurfaceVariant,
                ));
            }
            if let Some(icon) = trailing {
                item.spawn(quiet_slot(ColorRole::OnSurfaceVariant))
                    .with_child(crate::icon::icon(icon, crate::icon::ICON));
            }
        })),
    )
}

/// Opens, closes and returns the focus for a menu, as its button and its
/// popup ask.
fn on_menu_event(
    mut event: On<MenuEvent>,
    children: Query<&Children>,
    popups: Query<&Visibility, With<MenuSurface>>,
    openers: Query<(), With<MenuOpener>>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let menu = event.source;
    let Ok(parts) = children.get(menu) else {
        return;
    };
    let popup = parts.iter().find(|part| popups.contains(*part));
    let opener = parts.iter().find(|part| openers.contains(*part));
    match event.action {
        MenuAction::Open(nav) => {
            if let Some(popup) = popup {
                event.propagate(false);
                commands
                    .entity(popup)
                    .try_insert((Visibility::Visible, MenuFocusState::Opening(nav)));
            }
        }
        MenuAction::Toggle => {
            if let Some(popup) = popup {
                event.propagate(false);
                if matches!(popups.get(popup), Ok(Visibility::Visible)) {
                    commands.entity(popup).try_insert(Visibility::Hidden);
                } else {
                    commands.entity(popup).try_insert((
                        Visibility::Visible,
                        MenuFocusState::Opening(NavAction::First),
                    ));
                }
            }
        }
        MenuAction::CloseAll => {
            if let Some(popup) = popup {
                event.propagate(false);
                commands.entity(popup).try_insert(Visibility::Hidden);
            }
        }
        MenuAction::FocusRoot => {
            if let Some(opener) = opener {
                event.propagate(false);
                focus.set(opener, FocusCause::Navigated);
            }
        }
    }
}

/// When an item is chosen, sends its index on its menu, and shows it in its
/// select field if it is one.
fn on_choice(
    activate: On<Activate>,
    choices: Query<&MenuChoice>,
    mut menus: Query<Option<&mut SelectField>, With<MaterialMenuRoot>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Ok(MenuChoice(index)) = choices.get(activate.entity) else {
        return;
    };
    let Some(menu) = parents
        .iter_ancestors(activate.entity)
        .find(|ancestor| menus.contains(*ancestor))
    else {
        return;
    };
    if let Ok(Some(mut field)) = menus.get_mut(menu) {
        field.selected = Some(*index);
    }
    commands.trigger(ValueChange {
        source: menu,
        value: *index,
        is_final: true,
    });
}

/// A select field's options and the one chosen; change `selected` to show
/// another.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SelectField {
    /// The options, in order.
    pub options: Vec<String>,
    /// The index of the one chosen, if one is.
    pub selected: Option<usize>,
}

impl SelectField {
    /// The words of the chosen option, or none.
    pub fn chosen(&self) -> Option<&str> {
        self.selected
            .and_then(|index| self.options.get(index))
            .map(String::as_str)
    }
}

/// The words in a select field showing the chosen option.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SelectValue;

/// A select field: its label, options and the one chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialSelect {
    /// The words naming what it chooses.
    pub label: String,
    /// The options.
    pub options: Vec<String>,
    /// The one chosen, if one is.
    pub selected: Option<usize>,
    /// Whether it can be used.
    pub enabled: bool,
}

/// A select field named `label`, choosing one of `options`, none chosen yet.
pub fn select(
    label: impl Into<String>,
    options: impl IntoIterator<Item = impl Into<String>>,
) -> MaterialSelect {
    MaterialSelect {
        label: label.into(),
        options: options.into_iter().map(Into::into).collect(),
        selected: None,
        enabled: true,
    }
}

impl MaterialSelect {
    /// With the option at `index` chosen.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let MaterialSelect {
            label: name,
            options,
            selected,
            enabled,
        } = self;
        let field = SelectField {
            options: options.clone(),
            selected,
        };
        let shown = field.chosen().unwrap_or_default().to_owned();
        (
            MaterialMenuRoot,
            field,
            observe(on_menu_event),
            Node {
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(210.0),
                ..default()
            },
            Children::spawn(SpawnWith(move |root: &mut ChildSpawner| {
                root.spawn((
                    (
                        MenuButton,
                        MenuOpener,
                        Look::bare(ColorRole::OnSurface),
                        Stroke::new(ColorRole::Outline),
                        TabIndex(0),
                        FocusRing,
                        crate::button::enabled(enabled),
                    ),
                    Node {
                        height: Val::Px(56.0),
                        padding: UiRect::new(Val::Px(16.0), Val::Px(12.0), Val::ZERO, Val::ZERO),
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: Corner::ExtraSmall.all(),
                        ..default()
                    },
                ))
                .with_children(|field| {
                    field
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|words| {
                            words.spawn(quiet_text(
                                name,
                                TypeRole::BodySmall,
                                ColorRole::OnSurfaceVariant,
                            ));
                            words.spawn((SelectValue, label(shown, TypeRole::BodyLarge)));
                        });
                    field
                        .spawn(quiet_slot(ColorRole::OnSurfaceVariant))
                        .with_child(crate::icon::icon(Glyph::ChevronDown, crate::icon::ICON));
                });
                root.spawn(popup(Val::Percent(100.0)))
                    .with_children(|popup| {
                        let lines = options
                            .into_iter()
                            .enumerate()
                            .map(|(index, option)| {
                                MenuLine::Item(menu_item(option).selected(Some(index) == selected))
                            })
                            .collect();
                        spawn_lines(popup, lines);
                    });
            })),
        )
    }
}

/// Shows each select field's chosen option, in its words and its menu.
fn draw_selects(
    fields: Query<(Entity, &SelectField), Changed<SelectField>>,
    children: Query<&Children>,
    mut values: Query<&mut Text, With<SelectValue>>,
    mut items: Query<(&MenuChoice, &mut Look)>,
) {
    for (entity, field) in &fields {
        for part in children.iter_descendants(entity) {
            if let Ok(mut text) = values.get_mut(part) {
                let chosen = field.chosen().unwrap_or_default();
                if text.0 != chosen {
                    chosen.clone_into(&mut text.0);
                }
            }
            if let Ok((MenuChoice(index), mut look)) = items.get_mut(part) {
                look.set_if_neq(item_look(field.selected == Some(*index)));
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_choice)
        .add_systems(PostUpdate, draw_selects.before(crate::PaintSystems));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_select_field_shows_the_words_of_its_chosen_option() {
        let mut field = SelectField {
            options: vec!["Low".into(), "High".into()],
            selected: None,
        };
        assert_eq!(field.chosen(), None);
        field.selected = Some(1);
        assert_eq!(field.chosen(), Some("High"));
        field.selected = Some(7);
        assert_eq!(
            field.chosen(),
            None,
            "an index past the options shows nothing"
        );
    }

    #[test]
    fn a_menus_lines_keep_their_order_and_its_chosen_item_is_on_the_secondary_container() {
        let built = menu(crate::button("Edit"))
            .item(menu_item("Cut").shortcut("Ctrl+X"))
            .divider()
            .item(menu_item("Paste").enabled(false));
        assert_eq!(built.lines.len(), 3);
        assert_eq!(built.lines[1], MenuLine::Divider);
        assert_eq!(
            item_look(true).container,
            Some(ColorRole::SecondaryContainer)
        );
        assert_eq!(item_look(false).container, None);
    }
}

//! An interface laid out for real by Bevy, with no window or GPU: for the
//! tests of this crate and of a game, that check where parts land, what
//! colour they are drawn, and what happens when the pointer or the keyboard
//! uses them.

use bevy::camera::{Camera, Camera2d, ComputedCameraValues, RenderTargetInfo, Viewport};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{FocusCause, InputFocus, InputFocusVisible};
use bevy::picking::pointer::{Location, PointerAction, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowRef, WindowResolution};

/// Window sizes in logical pixels, with the display's scale factor: small,
/// common and large, wide and tall.
pub const WINDOWS: &[(u32, u32, f32)] = &[
    (640, 360, 1.0),
    (800, 600, 1.0),
    (1280, 720, 1.0),
    (1280, 720, 1.5),
    (1920, 1080, 1.0),
    (900, 1200, 1.0),
];

/// An app that lays its interface out as Bevy does in a game, in a window
/// `width` x `height` logical pixels at `scale`, with Material's components
/// ([`crate::MaterialUiPlugin`]). Its frames take a sixtieth of a second
/// however busy the machine, so what is timed - a snackbar, a tooltip's
/// delay - runs the same in every run.
pub fn headless_app(width: u32, height: u32, scale: f32) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::image::TextureAtlasPlugin,
        TransformPlugin,
        bevy::input::InputPlugin,
        WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        },
        bevy::a11y::AccessibilityPlugin,
        DefaultPickingPlugins,
        bevy::text::TextPlugin,
        bevy::ui::UiPlugin,
        crate::MaterialUiPlugin,
    ))
    // Picking skips a node that isn't visible, which this works out.
    .add_systems(
        PostUpdate,
        propagate_visibility
            .in_set(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
    )
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "window sizes in the thousands, scales of 1 or 1.5"
    )]
    let physical = UVec2::new(
        (width as f32 * scale).round() as u32,
        (height as f32 * scale).round() as u32,
    );
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(physical.x, physical.y)
                .with_scale_factor_override(scale),
            ..default()
        },
        PrimaryWindow,
    ));
    app.world_mut().spawn((
        Camera2d,
        Camera {
            computed: ComputedCameraValues {
                target_info: Some(RenderTargetInfo {
                    physical_size: physical,
                    scale_factor: scale,
                }),
                ..default()
            },
            viewport: Some(Viewport {
                physical_size: physical,
                ..default()
            }),
            ..default()
        },
    ));
    app.update();
    app
}

/// Spawns `bundle` in a column filling the window, lays it out, and returns
/// it.
pub fn spawn(app: &mut App, bundle: impl Bundle) -> Entity {
    let root = app
        .world_mut()
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            bevy::input_focus::tab_navigation::TabGroup::new(0),
        ))
        .id();
    let entity = app.world_mut().spawn((bundle, ChildOf(root))).id();
    settle(app);
    entity
}

/// Runs frames until what was spawned is laid out, text included.
pub fn settle(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}

/// A node's rectangle on screen, in physical pixels.
pub fn rect(app: &App, entity: Entity) -> Rect {
    let world = app.world();
    let node = world.get::<ComputedNode>(entity).expect("a laid-out node");
    let transform = world
        .get::<UiGlobalTransform>(entity)
        .expect("a laid-out node");
    Rect::from_center_size(transform.translation, node.size)
}

/// Whether `inner` lies within `outer`, give or take a pixel's rounding.
pub fn within(inner: Rect, outer: Rect) -> bool {
    const ROUNDING: f32 = 1.0;
    inner.min.x >= outer.min.x - ROUNDING
        && inner.min.y >= outer.min.y - ROUNDING
        && inner.max.x <= outer.max.x + ROUNDING
        && inner.max.y <= outer.max.y + ROUNDING
}

/// Everything below `entity`, at any depth, depth first.
pub fn descendants(app: &App, entity: Entity) -> Vec<Entity> {
    let mut found = Vec::new();
    let mut stack = vec![entity];
    while let Some(parent) = stack.pop() {
        if let Some(children) = app.world().get::<Children>(parent) {
            found.extend(children.iter());
            stack.extend(children.iter().rev());
        }
    }
    found
}

/// Every word of text at or below `entity`, joined by spaces.
pub fn words(app: &App, entity: Entity) -> String {
    std::iter::once(entity)
        .chain(descendants(app, entity))
        .filter_map(|part| app.world().get::<Text>(part))
        .map(|text| text.0.clone())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The window's rectangle, in physical pixels.
pub fn screen(app: &mut App) -> Rect {
    let window = window_of(app);
    let window = app.world().get::<Window>(window).expect("a window");
    Rect::new(
        0.0,
        0.0,
        window.physical_width() as f32,
        window.physical_height() as f32,
    )
}

fn window_of(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .expect("one window")
}

/// Sends the mouse's `action` at `at`, in logical pixels.
fn pointer(app: &mut App, at: Vec2, action: PointerAction) {
    let window = window_of(app);
    let location = Location {
        target: bevy::camera::NormalizedRenderTarget::Window(
            WindowRef::Entity(window)
                .normalize(Some(window))
                .expect("the primary window"),
        ),
        position: at,
    };
    app.world_mut()
        .write_message(PointerInput::new(PointerId::Mouse, location, action));
}

/// The centre of a node, in logical pixels.
pub fn centre(app: &mut App, entity: Entity) -> Vec2 {
    let scale = app
        .world()
        .get::<ComputedNode>(entity)
        .expect("a laid-out node")
        .inverse_scale_factor;
    rect(app, entity).center() * scale
}

/// Moves the mouse over `entity`'s centre, scrolled into view, and runs a
/// frame.
pub fn hover(app: &mut App, entity: Entity) {
    scroll_into_view(app, entity);
    let at = centre(app, entity);
    pointer(app, at, PointerAction::Move { delta: Vec2::ZERO });
    app.update();
}

/// Scrolls `entity` into view in the scroll area it is in, as a player
/// would before using it, and lays the page out again.
pub fn scroll_into_view(app: &mut App, entity: Entity) {
    app.world_mut()
        .trigger(bevy::ui_widgets::ScrollIntoView { entity });
    settle(app);
}

/// Clicks `entity` with the mouse - scrolled into view, moved over,
/// pressed and released - and runs frames until the click has done what it
/// does.
pub fn click(app: &mut App, entity: Entity) {
    scroll_into_view(app, entity);
    let at = centre(app, entity);
    click_at(app, at);
}

/// Clicks at `at`, in logical pixels.
pub fn click_at(app: &mut App, at: Vec2) {
    use bevy::picking::pointer::PointerButton;
    pointer(app, at, PointerAction::Move { delta: Vec2::ZERO });
    app.update();
    pointer(app, at, PointerAction::Press(PointerButton::Primary));
    app.update();
    pointer(app, at, PointerAction::Release(PointerButton::Primary));
    app.update();
    app.update();
}

/// Gives `entity` the keyboard's focus, shown as by the Tab key.
pub fn focus(app: &mut App, entity: Entity) {
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(entity, FocusCause::Navigated);
    if let Some(mut visible) = app.world_mut().get_resource_mut::<InputFocusVisible>() {
        visible.0 = true;
    }
    app.update();
}

/// Presses and releases `key`, and runs frames until it has done what it
/// does.
pub fn press(app: &mut App, key_code: KeyCode, logical_key: Key) {
    let window = window_of(app);
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: logical_key.clone(),
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }
}

/// Runs frames for `seconds` of the app's time.
pub fn wait(app: &mut App, seconds: f32) {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a test waits seconds, not years"
    )]
    let frames = (seconds * 60.0).ceil() as u32;
    for _ in 0..frames {
        app.update();
    }
}

/// The first entity with component `T`, at or below `root`.
pub fn find<T: Component>(app: &App, root: Entity) -> Option<Entity> {
    std::iter::once(root)
        .chain(descendants(app, root))
        .find(|entity| app.world().get::<T>(*entity).is_some())
}

/// Every entity with component `T`, at or below `root`.
pub fn find_all<T: Component>(app: &App, root: Entity) -> Vec<Entity> {
    std::iter::once(root)
        .chain(descendants(app, root))
        .filter(|entity| app.world().get::<T>(*entity).is_some())
        .collect()
}

/// Works out which nodes are visible, as Bevy's visibility plugin does in
/// a game: a node is hidden if it or an ancestor is `Hidden`, unless it is
/// `Visible` itself.
fn propagate_visibility(
    roots: Query<Entity, (With<Node>, Without<ChildOf>)>,
    children: Query<&Children>,
    mut nodes: Query<(Option<&Visibility>, &mut InheritedVisibility)>,
) {
    let mut stack: Vec<(Entity, bool)> = roots.iter().map(|root| (root, true)).collect();
    while let Some((entity, parent_visible)) = stack.pop() {
        let visible = match nodes.get_mut(entity) {
            Ok((visibility, mut inherited)) => {
                let visible = match visibility {
                    Some(Visibility::Hidden) => false,
                    Some(Visibility::Visible) => true,
                    _ => parent_visible,
                };
                if inherited.get() != visible {
                    *inherited = if visible {
                        InheritedVisibility::VISIBLE
                    } else {
                        InheritedVisibility::HIDDEN
                    };
                }
                visible
            }
            Err(_) => parent_visible,
        };
        if let Ok(kids) = children.get(entity) {
            stack.extend(kids.iter().map(|kid| (kid, visible)));
        }
    }
}
/// Drags with the mouse from `from` to `to`, in logical pixels: pressed,
/// moved there in a few steps, and released, a frame apart.
pub fn drag(app: &mut App, from: Vec2, to: Vec2) {
    use bevy::picking::pointer::PointerButton;
    const STEPS: u8 = 5;
    pointer(app, from, PointerAction::Move { delta: Vec2::ZERO });
    app.update();
    pointer(app, from, PointerAction::Press(PointerButton::Primary));
    app.update();
    let mut at = from;
    for step in 1..=STEPS {
        let next = from.lerp(to, f32::from(step) / f32::from(STEPS));
        pointer(app, next, PointerAction::Move { delta: next - at });
        at = next;
        app.update();
    }
    pointer(app, to, PointerAction::Release(PointerButton::Primary));
    app.update();
    app.update();
}

/// Types `words` into whatever holds the keyboard's focus, a key at a time,
/// and runs frames until the edits are applied.
pub fn type_text(app: &mut App, words: &str) {
    let window = window_of(app);
    for character in words.chars() {
        let key = if character == ' ' {
            Key::Space
        } else {
            Key::Character(character.to_string().into())
        };
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(KeyboardInput {
                key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                logical_key: key.clone(),
                state,
                text: (state == ButtonState::Pressed).then(|| character.to_string().into()),
                repeat: false,
                window,
            });
            app.update();
        }
    }
    app.update();
}

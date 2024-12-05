//! Playground
use bevy::{
    color::palettes::{css::WHITE, tailwind::*},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cant_wait_for_bsn::{Scene, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, game_health_update)
        .run();
}

#[derive(Component, Default, Clone)]
struct Health {
    current: i32,
    max: i32,
}

fn game_health_update(mut players: Query<&mut Health>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    let mut health = players.iter_mut().next().unwrap();
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        health.current = (health.current - 100).clamp(0, health.max);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        health.current = (health.current + 100).clamp(0, health.max);
    }
}

fn setup(mut commands: Commands) {
    // UI Camera
    commands.spawn((Camera2d, IsDefaultUiCamera));

    // Spawn players
    commands.spawn_scene(player("Player1"));
    commands.spawn_scene(player("Player2"));

    // UI root
    commands.spawn_scene(ui());
}

fn player(name: &'static str) -> impl Scene {
    bsn! {(
        {Name::new(name)},
        Health {
            current: 2000,
        },
        :max_health(3000),
    )}
}

fn max_health(max: i32) -> impl Scene {
    bsn! {(
        Health {
            max: max,
        },
    )}
}

fn ui() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        } [
            HealthBar {
                player_entity: @"Player1",
            }
        ]
    }
}

#[derive(Component, Clone)]
struct HealthBar {
    player_entity: ConstructEntity,
}

#[allow(missing_docs)]
#[derive(Clone)]
pub struct HealthBarProps {
    player_entity: ConstructProp<ConstructEntity>,
}

impl Default for HealthBarProps {
    fn default() -> Self {
        Self {
            player_entity: ConstructProp::Prop(Default::default()),
        }
    }
}

impl Construct for HealthBar {
    type Props = HealthBarProps;
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        let player_entity = props.player_entity.construct(context)?;

        let health = context
            .world
            .query::<&Health>()
            .get(context.world, *player_entity)
            .ok();

        let text = health
            .map(|h| format!("{}/{}", h.current, h.max))
            .unwrap_or("-".to_string());

        let normalized = health
            .map(|h| (h.current as f32) / (h.max as f32))
            .unwrap_or(0.0);

        let bar_color = if normalized < 0.2 {
            RED_500
        } else if normalized < 0.8 {
            YELLOW_500
        } else {
            LIME_500
        };

        let border_color = if normalized < 0.2 { RED_300 } else { GRAY_100 };

        let width = 250.0f32;
        let border = 3.0f32;
        let border_radius = px(10.0f32);
        let border_radius2 = px(7.0f32);
        let bar_right = width - ((width - (border * 2.0)) * normalized);

        let font_handle = context.construct::<ConstructHandle<Font>>("fonts/FiraSans-Bold.ttf")?;

        let entity_patch = bsn! {
            (
                Node {
                    width: px(width),
                    height: px(50.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: px_all(border),
                },
                BackgroundColor(GRAY_100),
                BorderColor(border_color),
                BorderRadius {
                    bottom_left: border_radius,
                    bottom_right: border_radius,
                    top_left: border_radius,
                    top_right: border_radius,
                },
            ) [
                (
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::ZERO,
                        bottom: px(border * 2.0),
                        left: Val::ZERO,
                        right: px(bar_right),
                    },
                    BackgroundColor(bar_color),
                    BorderRadius {
                        bottom_left: border_radius2,
                        bottom_right: border_radius2,
                        top_left: border_radius2,
                        top_right: border_radius2,
                    },
                ),
                (
                    Text(text.clone()),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 40.0,
                    },
                    TextColor(WHITE),
                )
            ]
        };

        context.construct_entity_patch(entity_patch)?;

        Ok(Self { player_entity })
    }
}

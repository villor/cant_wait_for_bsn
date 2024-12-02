//! Playground
use bevy::{
    color::palettes::{css::WHITE, tailwind::*},
    prelude::*,
};
use cant_wait_for_bsn::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, game_health_update)
        .run();
}

#[derive(Component, Reflect)]
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
    let player1 = commands
        .spawn(Health {
            current: 2000,
            max: 2000,
        })
        .id();

    // UI Camera
    commands.spawn((Camera2d, IsDefaultUiCamera));

    // UI root
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.construct::<HealthBar>(HealthBarProps {
                player_entity: ConstructProp::Prop(player1.into()),
            });
        });
}

#[derive(Deref, Clone)]
struct EntityRef(Entity);

#[derive(Default, Clone)]
enum EntityPath {
    #[default]
    None,
    Name(String),
    Entity(Entity),
}

impl From<String> for EntityPath {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

impl From<Entity> for EntityPath {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl Construct for EntityRef {
    type Props = EntityPath;

    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        match props {
            EntityPath::Name(name) => {
                let mut query = context.world.query::<(Entity, &Name)>();
                let entity = query
                    .iter(context.world)
                    .filter(|(_, q_name)| q_name.as_str() == name)
                    .map(|(entity, _)| EntityRef(entity))
                    .next();

                entity.ok_or_else(|| ConstructError::InvalidProps {
                    message: format!("entity with name {} does not exist", name).into(),
                })
            }
            EntityPath::Entity(entity) => Ok(EntityRef(entity)),
            _ => Err(ConstructError::InvalidProps {
                message: "no entity supplied".into(),
            }),
        }
    }
}

#[derive(Component, Clone)]
struct HealthBar {
    player_entity: EntityRef,
}

#[allow(missing_docs)]
#[derive(Clone)]
pub struct HealthBarProps {
    player_entity: ConstructProp<EntityRef>,
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
            .get(context.world, player_entity.0)
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
        let border_radius = Val::Px(10.0f32);
        let border_radius2 = Val::Px(7.0f32);
        let bar_right = width - ((width - (border * 2.0)) * normalized);

        let font_handle = context.construct::<Handle<Font>>("fonts/FiraSans-Bold.ttf")?;

        let entity_patch = bsn! {
            (
                Node {
                    width: Val::Px(width),
                    height: Val::Px(50.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(border)),
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
                        bottom: Val::Px(border * 2.0),
                        left: Val::ZERO,
                        right: Val::Px(bar_right),
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

        context.spawn_entity_patch(entity_patch)?;

        // let entity_patch = EntityPatch {
        //     patch: (
        //         Node::patch(move |props| {
        //             props.width = Val::Px(width);
        //             props.height = Val::Px(50.0);
        //             props.align_items = AlignItems::Center;
        //             props.justify_content = JustifyContent::Center;
        //             props.border = UiRect::all(Val::Px(border));
        //         }),
        //         BackgroundColor::patch(|props| props.0 = GRAY_800.into()),
        //         BorderColor::patch(move |props| props.0 = border_color.into()),
        //         BorderRadius::patch(|props| {
        //             let val = Val::Px(10.0);
        //             props.bottom_left = val;
        //             props.bottom_right = val;
        //             props.top_left = val;
        //             props.top_right = val;
        //         }),
        //     ),
        //     children: (
        //         EntityPatch {
        //             patch: (
        //                 Node::patch(move |props| {
        //                     props.position_type = PositionType::Absolute;
        //                     props.top = Val::ZERO;
        //                     props.bottom = Val::Px(border * 2.0);
        //                     props.left = Val::ZERO;
        //                     props.right = Val::Px(bar_right);
        //                 }),
        //                 BackgroundColor::patch(move |props| props.0 = bar_color.into()),
        //                 BorderRadius::patch(|props| {
        //                     let val = Val::Px(7.0);
        //                     props.bottom_left = val;
        //                     props.bottom_right = val;
        //                     props.top_left = val;
        //                     props.top_right = val;
        //                 }),
        //             ),
        //             children: (),
        //         },
        //         EntityPatch {
        //             patch: (
        //                 Text::patch(move |props| props.0 = text.clone()),
        //                 TextFont::patch(move |props| {
        //                     props.font = font_handle.clone();
        //                     props.font_size = 40.0;
        //                 }),
        //                 TextColor::patch(|props| props.0 = WHITE.into()),
        //             ),
        //             children: (),
        //         },
        //     ),
        // };

        //let mut entity = context.world.entity_mut(context.id);

        // entity
        //     .insert((
        //         Node {
        //             width: Val::Px(width),
        //             height: Val::Px(50.0),
        //             align_items: AlignItems::Center,
        //             justify_content: JustifyContent::Center,
        //             border: UiRect::all(Val::Px(border)),
        //             ..default()
        //         },
        //         BackgroundColor(GRAY_800.into()),
        //         BorderColor(border_color.into()),
        //         BorderRadius::all(Val::Px(10.0)),
        //     ))
        //     .with_children(|parent| {
        //         parent.spawn((
        //             Node {
        //                 position_type: PositionType::Absolute,
        //                 top: Val::ZERO,
        //                 bottom: Val::Px(border * 2.0), // weird stuff, bug?
        //                 left: Val::ZERO,
        //                 right: Val::Px(bar_right),
        //                 ..default()
        //             },
        //             BackgroundColor(bar_color.into()),
        //             BorderRadius::all(Val::Px(7.0)),
        //         ));
        //         parent.spawn((
        //             Text::new(text),
        //             TextFont {
        //                 font: font_handle,
        //                 font_size: 40.0,
        //                 ..default()
        //             },
        //             TextColor(WHITE.into()),
        //         ));
        //     });

        Ok(Self { player_entity })
    }
}

//! Playground
use bevy::{color::palettes::tailwind::*, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cant_wait_for_bsn::{Scene, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn_scene(ui());
        })
        .add_systems(Update, update_button_background)
        .add_systems(Update, update_button_font)
        .run();
}

fn ui() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(5.0),
        } [
            (Node, :button("Basic")),
            (Node, :button("Rounded"), rounded),
            (Node { border: px_all(5.0) }, BorderColor(RED_500) :button("Thick red"), rounded),
            (Node, :button("Merged children"), rounded) [(
                Node {
                    width: px(30.0),
                    height: px(30.0),
                },
                BackgroundColor(BLUE_500),
                {BorderRadius::MAX}
            )],
        ]
    }
}

fn button(text: &'static str) -> impl Scene {
    bsn! {(
        Button,
        Node {
            padding: px_all(5.0),
            border: px_all(2.0),
            align_items: AlignItems::Center,
            column_gap: px(3.0),
        },
        BorderColor(LIME_800),
        BackgroundColor(LIME_500)
    ) [
        (
            Text(text),
            ConstructableTextFont {
                font: @"fonts/FiraSans-bold.ttf"
            }
        )
    ]}
}

fn update_button_background(
    mut commands: Commands,
    query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    for (entity, interaction) in query.iter() {
        match *interaction {
            Interaction::Pressed => commands
                .entity(entity)
                .construct_patch(bsn! { BackgroundColor(LIME_600) }),
            Interaction::Hovered => commands
                .entity(entity)
                .construct_patch(bsn! { BackgroundColor(LIME_400) }),
            Interaction::None => commands
                .entity(entity)
                .construct_patch(bsn! { BackgroundColor(LIME_500) }),
        };
    }
}

fn update_button_font(
    mut commands: Commands,
    query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<Entity, With<Text>>,
) {
    for (interaction, children) in query.iter() {
        let entity = text_query.get(children[0]).unwrap();
        let font = match *interaction {
            Interaction::Pressed => "fonts/Comic Sans.ttf",
            _ => "fonts/FiraSans-Bold.ttf",
        };
        commands.entity(entity).construct_patch(bsn! {
            ConstructableTextFont {
                font: @font,
            }
        });
    }
}

fn rounded() -> impl Scene {
    bsn! {(
        {BorderRadius::all(px(10.0))}
    )}
}

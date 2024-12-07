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
        {Text::new(text)}
    ]}
}

fn rounded() -> impl Scene {
    bsn! {(
        {BorderRadius::all(px(10.0))}
    )}
}

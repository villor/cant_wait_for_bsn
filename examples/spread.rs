//! Spread operator ..
use bevy::{color::palettes::tailwind::*, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cant_wait_for_bsn::{Scene, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn_scene(ui_root()).with_children(|parent| {
                parent.spawn_scene(list());
            });
        })
        .run();
}

fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(5.0),
        }
    }
}

fn list() -> impl Scene {
    const NUMBERS: [&str; 5] = ["One", "Two", "Three", "Four", "Five"];

    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(5.0)
        } [
            Text("First child"),
            ..NUMBERS.iter().map(|text|
                bsn! {(
                    TextColor(ORANGE_500),
                    Text(*text),
                )}
            ),
            Text("Last child"),
        ]
    }
}

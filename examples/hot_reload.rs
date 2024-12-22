//! Hot reload
use bevy::{color::palettes::tailwind::*, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cant_wait_for_bsn::{Scene, *};

fn main() {
    App::new()
        .register_bsn_hot_reload_source("examples")
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CantWaitForBsnPlugin)
        .add_plugins(BsnHotReloadPlugin)
        .add_systems(
            Startup,
            (|mut commands: Commands| {
                commands.spawn(Camera2d);
                commands.spawn_scene(ui());
            },),
        )
        .run();
}

fn ui() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(15.0),
        } [
            (
                Button,
                Node {
                    padding: px_all(5.0),
                },
                BorderColor(LIME_800),
                BackgroundColor(LIME_500),
            ) [
                Text("OK"),
            ],
            (
                Button,
                Node {
                    padding: px_all(5.0),
                },
                BorderColor(GRAY_800),
                BackgroundColor(GRAY_500),
            ) [
                Text("Cancel"),
            ]
        ]
    }
}

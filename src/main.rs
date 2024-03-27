extern crate bevy;
extern crate bevy_inspector_egui;
extern crate serde;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod ron_asset_macro;
mod types;
use types::{Card, CardAssetPlugin};
const WINDOW_SIZE: UVec2 = UVec2 { x: 426, y: 240 }; // 240p

#[derive(Component, Reflect)]
struct CardTest {
    pub _card: Handle<Card>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CardAssetPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, print_assets)
        .register_type::<CardTest>()
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let card: Handle<Card> = asset_server.load("cards/ostrich.card.ron");
    commands.spawn(CardTest { _card: card });

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WINDOW_SIZE[0] as f32,
                height: WINDOW_SIZE[1] as f32,
            },
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        }),
        ..Default::default()
    });
}

fn print_assets(assets: Res<Assets<Card>>) {
    println!("{:#?}", assets.len());
    for asset in assets.iter() {
        println!("{:#?}", asset.1)
    }
}

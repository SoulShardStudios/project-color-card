extern crate bevy;
extern crate bevy_inspector_egui;
extern crate serde;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod card;
mod ron_asset_macro;
use card::{Card, CardAssetPlugin};
const WINDOW_SIZE: UVec2 = UVec2 { x: 426, y: 240 }; // 240p

#[derive(Resource)]
struct CardTest {
    pub _card: UntypedHandle,
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
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CardTest {
        _card: asset_server.load_folder("cards").into(),
    });

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

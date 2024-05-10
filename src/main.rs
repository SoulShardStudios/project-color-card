extern crate bevy;
extern crate bevy_inspector_egui;
extern crate bevy_rand;
extern crate serde;
use assets::AssetLoaderPlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod cards;
mod ron_asset_macro;
use bevy_rand::prelude::{EntropyPlugin, WyRand};
mod game;
use game::GameUIPlugin;
mod assets;
mod custom_cursor;

#[macro_use]
extern crate num_derive;
fn main() {
    App::new()
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(GameUIPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

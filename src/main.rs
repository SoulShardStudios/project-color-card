extern crate bevy;
extern crate bevy_inspector_egui;
extern crate bevy_rand;
extern crate serde;
use assets::{AssetLoaderPlugin, LoadState};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod cards;
mod ron_asset_macro;
use bevy_rand::prelude::{EntropyPlugin, WyRand};
use bevy_rand::resource::GlobalEntropy;
mod main_ui;
use main_ui::{CardSlot, CardSlotType, GameUIController, GameUIPlugin};
mod assets;

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
        .add_systems(Update, (draw_cards).run_if(in_state(LoadState::Loaded)))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn draw_cards(mut game_ui: Query<&mut GameUIController>, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    let mut game_ui = match game_ui.iter_mut().nth(0) {
        None => {
            return;
        }
        Some(x) => x,
    };
    let card = game_ui.get_random_card(&mut rng).unwrap();
    game_ui.set_card(
        &CardSlot {
            id: 0,
            team: main_ui::Team::Blue,
            slot_type: CardSlotType::Play,
        },
        card,
    );
}

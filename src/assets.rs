use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::{ConfigureLoadingState, LoadingStateConfig};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

pub struct AssetLoaderPlugin;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum LoadState {
    #[default]
    Unloaded,
    Loaded,
}

// I would make an array here but handles stored in arrays or vecs do not stay active for some reason
#[derive(AssetCollection, Default, Resource)]
pub struct Assets {
    #[asset(path = "card_backs", collection)]
    pub card_backs: Vec<UntypedHandle>,
    #[asset(path = "cards", collection)]
    pub cards: Vec<UntypedHandle>,
}

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadState>()
            .add_loading_state(
                LoadingState::new(LoadState::Unloaded).continue_to_state(LoadState::Loaded),
            )
            .configure_loading_state(
                LoadingStateConfig::new(LoadState::Unloaded).load_collection::<Assets>(),
            );
    }
}

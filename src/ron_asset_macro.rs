#[macro_export]
macro_rules! create_ron_nested_asset_loader {
    ($loader_name: ident, $asset_name:ident, $extensions: expr, $asset_plugin_name: ident, $($string_name: ident -> $handle_name: ident)*, $mod_name: ident) => {
        struct $loader_name;
        pub struct $asset_plugin_name;

        mod $mod_name {
            use super::*;
            #[derive(Serialize, Deserialize, Default)]
            pub struct EmptySettings;

            fn finalize_assets(
                mut asset_events: EventReader<AssetEvent<$asset_name>>,
                mut assets: ResMut<Assets<$asset_name>>,
                asset_server: Res<AssetServer>,
            ) {
                for event in asset_events.read() {
                    match event {
                        AssetEvent::LoadedWithDependencies { id } => {
                            let x = assets.get_mut(*id).unwrap();
                            $(
                                x.$handle_name = asset_server.load(&x.$string_name);
                            )*
                        }
                        _default => {}
                    }
                }
            }

            impl AssetLoader for $loader_name {
                type Asset = $asset_name;
                type Error = std::io::Error;
                type Settings = EmptySettings;

                fn load<'a>(
                    &'a self,
                    reader: &'a mut Reader,
                    _settings: &'a EmptySettings,
                    _load_context: &'a mut LoadContext,
                ) -> BoxedFuture<'a, Result<$asset_name, Self::Error>> {
                    use bevy::asset::{ron, AsyncReadExt};
                    Box::pin(async move {
                        let mut bytes = Vec::new();
                        reader.read_to_end(&mut bytes).await?;
                        return match ron::de::from_bytes(&bytes) {
                            Err(_err) => Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                _err,
                            )),
                            Ok(ok) => Ok(ok),
                        };
                    })
                }

                fn extensions(&self) -> &[&str] {
                    $extensions
                }
            }



            impl Plugin for $asset_plugin_name {
                fn build(&self, app: &mut App) {
                    app.init_asset::<$asset_name>()
                        .register_asset_loader($loader_name).add_systems(Update, finalize_assets);
                }
            }
        }
    };
}

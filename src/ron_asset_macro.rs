use bevy::asset::{ron, transformer};
#[macro_export]
macro_rules! create_ron_asset_loader {
    ($loader_name: ident, $asset_name:ident, $settings_name: ident, $module_name: ident, $extensions: expr, $asset_plugin_name: ident) => {
        impl AssetLoader for $loader_name {
            type Asset = $asset_name;
            type Error = std::io::Error;
            type Settings = $settings_name;

            fn load<'a>(
                &'a self,
                reader: &'a mut Reader,
                _settings: &'a $settings_name,
                _load_context: &'a mut LoadContext,
            ) -> BoxedFuture<'a, Result<$asset_name, Self::Error>> {
                Box::pin(async move {
                    let mut bytes = Vec::new();
                    reader.read_to_end(&mut bytes).await?;
                    return match ron::de::from_bytes(&bytes) {
                        Err(_err) => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "failed to parse ron object",
                        )),
                        Ok(ok) => Ok(ok),
                    };
                })
            }

            fn extensions(&self) -> &[&str] {
                $extensions
            }
        }

        pub struct $asset_plugin_name;

        impl Plugin for $asset_plugin_name {
            fn build(&self, app: &mut App) {
                app.init_asset::<$asset_name>()
                    .register_asset_loader($loader_name);
            }
        }
    };
}

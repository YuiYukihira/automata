use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};

pub struct TxtAssetPlugin;

impl Plugin for TxtAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<StringAsset>()
            .add_asset_loader(TxtAssetLoader);
    }
}

struct TxtAssetLoader;

impl AssetLoader for TxtAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let asset = String::from_utf8(bytes.to_vec())?;
            load_context.set_default_asset(LoadedAsset::new(StringAsset(asset)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(TypeUuid, Deref)]
#[uuid = "579f4885-5a11-46d3-a7e6-5528e254c836"]
struct StringAsset(String);

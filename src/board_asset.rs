use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};

pub(crate) struct BoardAssetPlugin;
impl Plugin for BoardAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BoardAsset>()
            .add_asset_loader(BoardAssetLoader);
    }
}

struct BoardAssetLoader;
impl AssetLoader for BoardAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let raw_data = String::from_utf8(bytes.to_vec())?;
            let board_data: Vec<Vec<bool>> = raw_data
                .lines()
                .map(|line| line.chars().map(|c| matches!(c, '■')).collect())
                .collect();
            let board_dimensions = (
                raw_data
                    .lines()
                    .fold(0, |acc, x| acc.max(x.chars().count())) as u32,
                raw_data.lines().count() as u32,
            );

            load_context.set_default_asset(LoadedAsset::new(BoardAsset {
                data: board_data,
                size: board_dimensions,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["board"]
    }
}

#[derive(TypeUuid)]
#[uuid = "579f4885-5a11-46d3-a7e6-5528e254c836"]
pub struct BoardAsset {
    pub data: Vec<Vec<bool>>,
    pub size: (u32, u32),
}

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

fn parse_cells(bytes: &[u8]) -> anyhow::Result<BoardAsset> {
    let raw_data = std::str::from_utf8(bytes)?;
    let board_data: Vec<Vec<bool>> = raw_data
        .lines()
        .filter(|l| !l.starts_with('!'))
        .map(|line| line.chars().map(|c| matches!(c, '■' | 'O')).collect())
        .collect();
    let board_dimensions = (
        raw_data
            .lines()
            .fold(0, |acc, x| acc.max(x.chars().count())) as u32,
        raw_data.lines().count() as u32,
    );

    Ok(BoardAsset {
        data: board_data,
        size: board_dimensions,
    })
}

struct BoardAssetLoader;
impl AssetLoader for BoardAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let board = match load_context.path().extension().unwrap().to_str().unwrap() {
                "rle" => {
                    let input = String::from_utf8(bytes.to_vec())?;
                    rle::parse(&input)?
                }
                "board" | "cells" => parse_cells(bytes)?,
                _ => unimplemented!(),
            };

            load_context.set_default_asset(LoadedAsset::new(board));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["board", "rle", "cells"]
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "579f4885-5a11-46d3-a7e6-5528e254c836"]
pub struct BoardAsset {
    pub data: Vec<Vec<bool>>,
    pub size: (u32, u32),
}

mod rle {
    use super::BoardAsset;

    enum State {
        Alive,
        Dead,
    }

    pub use parser::parse;

    peg::parser! {
        pub grammar parser() for str {
            rule _ -> ()
                = " " / "\t" / "\n" / "\r\n"

            rule number() -> usize
                = n:$(['0'..='9']+) {? n.parse().or(Err("usize")) }

            rule comment() -> ()
                = "#" [^ '\n']+ "\n"

            rule header_line() -> ()
                = [^ '\n']+ "\n"

            rule state() -> State
                = s:['o' | 'b'] { if s == 'o' {State::Alive} else {State::Dead} }

            rule rle_statement() -> (usize, State)
                = _* l:number()? s:state() _* { (l.unwrap_or(1), s) }

            rule line() -> Vec<bool>
                = xs:rle_statement()+ f:number()? ("$" / "!") { let mut v: Vec<_> = xs.into_iter().flat_map(|(count, state)| match state {
                    State::Alive => vec![true; count],
                    State::Dead => vec![false; count],
                }).collect();
                if let Some(f) = f {
                    v.append(&mut vec![false; f])
                }
                v}

            pub rule parse() -> BoardAsset
                = comment()* header_line() comment()* lines:line()+ "\n"? { let sizes = (
                    lines.iter().map(|l| l.len()).max().unwrap() as u32,
                    lines.len() as u32,
                );
                BoardAsset {
                    data: lines,
                    size: sizes
                } }
        }
    }
}

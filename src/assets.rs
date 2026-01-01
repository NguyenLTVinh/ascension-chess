use crate::types::*;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Assets {
    pub textures: HashMap<(PieceType, PlayerColor), Texture2D>,
    pub board_texture: Texture2D,
}

impl Assets {
    pub async fn load() -> Self {
        let mut textures = HashMap::new();

        let pieces = [
            (PieceType::Pawn, "P"),
            (PieceType::Knight, "N"),
            (PieceType::Bishop, "B"),
            (PieceType::Rook, "R"),
            (PieceType::Queen, "Q"),
            (PieceType::King, "K"),
            (PieceType::Hawk, "H"),
            (PieceType::Elephant, "E"),
            (PieceType::Archbishop, "A"),
            (PieceType::Cannon, "C"),
            (PieceType::Chancellor, "Ch"),
        ];

        for (pt, suffix) in pieces.iter() {
            let path = format!("assets/w{}.svg", suffix);
            if let Some(tex) = load_svg(&path, 80).await {
                textures.insert((*pt, PlayerColor::White), tex);
            } else {
                println!("Failed to load {}", path);
            }

            let path = format!("assets/b{}.svg", suffix);
            if let Some(tex) = load_svg(&path, 80).await {
                textures.insert((*pt, PlayerColor::Black), tex);
            } else {
                println!("Failed to load {}", path);
            }
        }

        let board_tex = load_svg("assets/board.svg", 640).await.unwrap_or_else(|| {
            println!("Failed to load board.svg");
            Texture2D::from_image(&Image::gen_image_color(640, 640, WHITE))
        });

        Self {
            textures,
            board_texture: board_tex,
        }
    }
}

async fn load_svg(path: &str, target_size: u32) -> Option<Texture2D> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return None,
    };

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&bytes, &opt).ok()?;

    let size = tree.size();
    let width = size.width();
    let height = size.height();
    let scale = target_size as f32 / width.max(height);

    let pixmap_size = tiny_skia::IntSize::from_wh((width * scale) as u32, (height * scale) as u32)?;
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

    let transform = tiny_skia::Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let image = Image {
        bytes: pixmap.data().to_vec(),
        width: pixmap_size.width() as u16,
        height: pixmap_size.height() as u16,
    };

    Some(Texture2D::from_image(&image))
}

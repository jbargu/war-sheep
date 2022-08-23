use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_graphics);
    }
}

const LETTER_TILE_WIDTH: f32 = 8.0;

/// Write Ascii text to the screen at `position`
pub fn write_text(
    commands: &mut Commands,
    texture: &AsciiSheet,
    translation: Vec3,
    color: Color,
    text: &str,
) -> Entity {
    let text_parent = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(translation),
            ..default()
        })
        .insert(Name::from(format!(
            "[TEXT] {}{}",
            text.chars().take(10).collect::<String>(),
            match text.chars().count().cmp(&10) {
                std::cmp::Ordering::Greater => "...",
                _ => "",
            }
        )))
        .id();

    let mut chars = Vec::with_capacity(text.len());
    for (row, line) in text.lines().enumerate() {
        for (col, char) in line.chars().enumerate() {
            chars.push(
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        transform: Transform::from_translation(
                            translation
                                + Vec3::X * col as f32 * LETTER_TILE_WIDTH / 16.0
                                + Vec3::Y * row as f32 * -LETTER_TILE_WIDTH / 16.0,
                        ),
                        texture_atlas: texture.0.clone(),
                        sprite: TextureAtlasSprite {
                            color,
                            index: char as usize,
                            custom_size: Some(Vec2::splat(LETTER_TILE_WIDTH) / 16.0),
                            ..default()
                        },
                        ..default()
                    })
                    .id(),
            );
        }
    }

    commands.entity(text_parent).push_children(&chars);
    text_parent
}

pub struct AsciiSheet(Handle<TextureAtlas>);

// https://dwarffortresswiki.org/Tileset_repository#Herrbdog_7x7_tileset.gif
// Licensed under GFDL & MIT
fn load_graphics(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("Ascii.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::new(LETTER_TILE_WIDTH, LETTER_TILE_WIDTH),
        16,
        16,
        Vec2::splat(2.0),
        Vec2::ZERO,
    );
    let atlas_handle = texture_atlases.add(atlas);
    commands.insert_resource(AsciiSheet(atlas_handle));
}

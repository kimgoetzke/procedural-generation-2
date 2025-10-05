use crate::components::{AnimationMeshComponent, AnimationType};
use crate::constants::*;
use crate::coords::Point;
use crate::coords::point::World;
use crate::generation::lib::{
  Chunk, ChunkComponent, GenerationResourcesCollection, Plane, TerrainType, Tile, TileMeshComponent, shared,
};
use crate::generation::resources::Metadata;
use crate::generation::world::post_processor;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::log::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::AlphaMode2d;
use std::collections::HashMap;

pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn generate_chunks(spawn_points: Vec<Point<World>>, metadata: Metadata, settings: &Settings) -> Vec<Chunk> {
  let start_time = shared::get_time();
  let mut chunks: Vec<Chunk> = Vec::new();
  for chunk_w in spawn_points {
    let chunk_tg = Point::new_tile_grid_from_world(chunk_w.clone());
    let mut chunk = Chunk::new(chunk_w.clone(), chunk_tg, &metadata, &settings);
    chunk = post_processor::process(chunk, &settings);
    chunks.push(chunk);
  }
  debug!(
    "Generated [{}] chunks in {} ms on {}",
    chunks.len(),
    shared::get_time() - start_time,
    shared::thread_name()
  );

  chunks
}

pub fn spawn_chunk(world_child_builder: &mut RelatedSpawnerCommands<ChildOf>, chunk: &Chunk) -> Entity {
  let chunk_end_tg = chunk.coords.tile_grid + Point::new(CHUNK_SIZE - 1, -CHUNK_SIZE + 1);
  world_child_builder
    .spawn((
      Name::new(format!(
        "Chunk {} {} {} to {}",
        chunk.coords.chunk_grid, chunk.coords.world, chunk.coords.tile_grid, chunk_end_tg
      )),
      Transform::default(),
      Visibility::default(),
      ChunkComponent {
        layered_plane: chunk.layered_plane.clone(),
        coords: chunk.coords.clone(),
      },
    ))
    .id()
}

pub fn spawn_tiles(
  commands: &mut Commands,
  chunk_entity: Entity,
  chunk: Chunk,
  settings: &Settings,
  resources: &GenerationResourcesCollection,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
) {
  let start_time = shared::get_time();
  let is_sprite_animation_disabled = !settings.general.animate_terrain_sprites;
  let is_drawing_terrain_sprites_disabled = !settings.general.draw_terrain_sprites;
  for layer in 0..TerrainType::length() {
    if layer < settings.general.spawn_from_layer || layer > settings.general.spawn_up_to_layer {
      trace!(
        "Skipped spawning [{:?}] tiles because it's disabled",
        TerrainType::from(layer)
      );
      continue;
    }

    if let Some(plane) = chunk.layered_plane.get(layer) {
      let texture_groups = prepare_texture_groups(resources, plane, is_drawing_terrain_sprites_disabled);
      for ((texture, has_animated_sprites, is_animated), tiles) in texture_groups {
        let is_animated = if is_drawing_terrain_sprites_disabled || is_sprite_animation_disabled {
          false
        } else {
          is_animated
        };
        spawn_tile_mesh(
          commands,
          resources,
          meshes,
          materials,
          tiles,
          texture,
          layer as f32,
          chunk_entity,
          has_animated_sprites,
          is_animated,
          is_drawing_terrain_sprites_disabled,
        );
      }
    }
  }

  debug!(
    "Created mesh(es) for chunk {} in {} ms on {}",
    chunk.coords.chunk_grid,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

/// The purpose of this function is to group tiles by their texture and whether they will be animated so that we can
/// spawn a single mesh for each texture.
fn prepare_texture_groups<'a>(
  resources: &GenerationResourcesCollection,
  plane: &'a Plane,
  is_drawing_terrain_sprites_disabled: bool,
) -> HashMap<(Handle<Image>, bool, bool), Vec<&'a Tile>> {
  let mut texture_groups: HashMap<(Handle<Image>, bool, bool), Vec<&Tile>> = HashMap::new();
  for row in plane.data.iter() {
    for tile in row.iter().flatten() {
      let asset_collection = resources.get_terrain_collection(&tile.terrain, &tile.climate);
      let has_animated_sprites = asset_collection.anim.is_some();
      let is_animated = asset_collection.animated_tile_types.contains(&tile.tile_type);
      let texture = match (is_drawing_terrain_sprites_disabled, has_animated_sprites) {
        (false, true) => {
          &asset_collection
            .anim
            .as_ref()
            .expect("Failed to get animated asset pack from resource collection")
            .texture
        }
        (false, false) => &asset_collection.stat.texture,
        (true, _) => &resources.placeholder.texture,
      };

      texture_groups
        .entry((texture.clone(), has_animated_sprites, is_animated))
        .or_default()
        .push(tile);
    }
  }

  texture_groups
}

fn spawn_tile_mesh(
  commands: &mut Commands,
  resources: &GenerationResourcesCollection,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
  tiles: Vec<&Tile>,
  texture: Handle<Image>,
  layer: f32,
  parent_entity: Entity,
  has_animated_sprites: bool,
  is_animated: bool,
  is_drawing_terrain_sprites_disabled: bool,
) {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
  let tiles_cloned = tiles.clone();
  let cg = tiles[0].coords.chunk_grid;
  let (vertices, indices, uvs, tile_sprite_indices, sprite_sheet_columns, sprite_sheet_rows) = calculate_mesh_attributes(
    &resources,
    tiles,
    layer,
    has_animated_sprites,
    is_drawing_terrain_sprites_disabled,
  );

  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
  mesh.insert_indices(Indices::U32(indices));

  commands.entity(parent_entity).with_children(|parent| {
    parent
      .spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial {
          alpha_mode: AlphaMode2d::Blend,
          texture: Some(texture),
          ..default()
        })),
        Transform::from_xyz(0.0, 0.0, layer),
        Name::new(format!("{:?} Mesh", TerrainType::from(layer as usize))),
        TileMeshComponent::new(parent_entity, cg, tiles_cloned.into_iter().copied().collect()),
      ))
      .insert_if(
        AnimationMeshComponent {
          animation_type: if TerrainType::from(layer as usize) == TerrainType::Shore {
            AnimationType::FourFramesDoubleSpeed
          } else {
            AnimationType::FourFramesDefaultSpeed
          },
          columns: sprite_sheet_columns,
          rows: sprite_sheet_rows,
          tile_indices: tile_sprite_indices,
        },
        || is_animated,
      );
  });
}

fn calculate_mesh_attributes(
  resources: &GenerationResourcesCollection,
  tiles: Vec<&Tile>,
  layer: f32,
  has_animated_sprites: bool,
  is_drawing_terrain_sprites_disabled: bool,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>, Vec<usize>, f32, f32) {
  let mut vertices = Vec::new();
  let mut indices = Vec::new();
  let mut uvs = Vec::new();
  let mut tile_indices = Vec::new();
  let tile_size = TILE_SIZE as f32;
  let columns = resolve_columns(has_animated_sprites, is_drawing_terrain_sprites_disabled);
  let rows = resolve_rows(is_drawing_terrain_sprites_disabled);

  for &tile in tiles {
    let sprite_index = resolve_sprite_index(resources, &tile, is_drawing_terrain_sprites_disabled);
    let base_idx = vertices.len() as u32;

    // Tile index for animation (ignored if not animated)
    tile_indices.push(sprite_index);

    // Calculate vertices
    let tile_x = tile.coords.world.x as f32;
    let tile_y = tile.coords.world.y as f32;
    vertices.push([tile_x, tile_y, layer]); // Top-left
    vertices.push([tile_x + tile_size, tile_y, layer]); // Top-right
    vertices.push([tile_x + tile_size, tile_y - tile_size, layer]); // Bottom-right
    vertices.push([tile_x, tile_y - tile_size, layer]); // Bottom-left

    // Calculate UVs
    let sprite_col = sprite_index as f32 % columns;
    let sprite_row = (sprite_index as f32 / columns).floor();
    let u_start = sprite_col / columns;
    let u_end = (sprite_col + 1.0) / columns;
    let v_start = sprite_row / rows;
    let v_end = (sprite_row + 1.0) / rows;
    uvs.push([u_start, v_start]); // Top-left
    uvs.push([u_end, v_start]); // Top-right
    uvs.push([u_end, v_end]); // Bottom-right
    uvs.push([u_start, v_end]); // Bottom-left

    // Add indices for both triangles
    indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);
  }

  (vertices, indices, uvs, tile_indices, columns, rows)
}

/// Determines the number of columns in the sprite sheet based on whether terrain sprites are disabled
/// and whether the asset collection contains animated sprites. The latter can only be true if terrain sprites
/// are enabled.
fn resolve_columns(has_animated_sprites: bool, is_drawing_terrain_sprites_disabled: bool) -> f32 {
  match (is_drawing_terrain_sprites_disabled, has_animated_sprites) {
    (true, _) => TILE_SET_PLACEHOLDER_COLUMNS as f32,
    (false, true) => DEFAULT_ANIMATED_TILE_SET_COLUMNS as f32,
    (false, false) => DEFAULT_STATIC_TILE_SET_COLUMNS as f32,
  }
}

/// Determines the number of rows in the sprite sheet based on whether terrain sprites are disabled.
fn resolve_rows(is_drawing_terrain_sprites_disabled: bool) -> f32 {
  if is_drawing_terrain_sprites_disabled {
    TILE_SET_PLACEHOLDER_ROWS as f32
  } else {
    TILE_SET_ROWS as f32
  }
}

/// Determines the sprite index for a tile based on its terrain, climate, and type. If drawing terrain sprites
/// is disabled, it simply returns the terrain type as the sprite index which corresponds to the placeholder sprite
/// sheet.
fn resolve_sprite_index(
  resources: &GenerationResourcesCollection,
  tile: &Tile,
  is_drawing_terrain_sprites_disabled: bool,
) -> usize {
  if is_drawing_terrain_sprites_disabled {
    return tile.terrain as usize;
  }

  tile
    .tile_type
    .calculate_sprite_index(&tile.terrain, &tile.climate, &resources)
}

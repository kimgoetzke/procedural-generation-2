use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::{shared, Chunk, ChunkComponent, TerrainType, Tile};
use crate::generation::resources::{GenerationResourcesCollection, Metadata};
use crate::generation::world::post_processor;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::asset::RenderAssetUsages;
use bevy::core::Name;
use bevy::hierarchy::{BuildChildren, ChildBuild, ChildBuilder};
use bevy::log::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::AlphaMode2d;
use std::collections::HashMap;

pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
  fn build(&self, _app: &mut App) {}
}

// TODO: Add tiles and use this struct for debugging
#[derive(Component)]
struct TileMeshComponent {
  parent_entity: Entity,
  is_animated: bool,
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
    "Generated {} chunks in {} ms on {}",
    chunks.len(),
    shared::get_time() - start_time,
    shared::thread_name()
  );

  chunks
}

pub fn spawn_chunk(world_child_builder: &mut ChildBuilder, chunk: &Chunk) -> Entity {
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

// TODO: Make settings.general.animate_terrain_sprites work again
pub fn spawn_tile_layer_meshes(
  commands: &mut Commands,
  settings: &Settings,
  chunk: Chunk,
  chunk_entity: Entity,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
  resources: &GenerationResourcesCollection,
) {
  let start_time = shared::get_time();
  for layer in 0..TerrainType::length() {
    if layer < settings.general.spawn_from_layer || layer > settings.general.spawn_up_to_layer {
      trace!(
        "Skipped spawning [{:?}] tiles because it's disabled",
        TerrainType::from(layer)
      );
      continue;
    }

    if let Some(plane) = chunk.layered_plane.get(layer) {
      let mut texture_groups: HashMap<(Handle<Image>, bool), Vec<&Tile>> = HashMap::new();
      for row in plane.data.iter() {
        for tile in row.iter().flatten() {
          let asset_collection = resources.get_terrain_collection(tile.terrain, tile.climate);
          let is_animated = asset_collection.anim.is_some();
          let texture = match is_animated {
            true => {
              &asset_collection
                .anim
                .as_ref()
                .expect("Failed to get animated asset pack from resource collection")
                .texture
            }
            false => &asset_collection.stat.texture,
          };
          texture_groups.entry((texture.clone(), is_animated)).or_default().push(tile);
        }
      }

      for ((texture, is_animated), tiles) in texture_groups {
        spawn_tile_mesh(
          commands,
          resources,
          meshes,
          materials,
          tiles,
          texture,
          layer as f32,
          chunk_entity,
          if is_animated { (4.0, 17.0) } else { (1.0, 17.0) },
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

fn spawn_tile_mesh(
  commands: &mut Commands,
  resources: &GenerationResourcesCollection,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
  tiles: Vec<&Tile>,
  texture: Handle<Image>,
  layer: f32,
  parent_entity: Entity,
  atlas_layout_dimensions: (f32, f32),
) {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
  let mut vertices = Vec::new();
  let mut indices = Vec::new();
  let mut uvs = Vec::new();
  let tile_size = TILE_SIZE as f32;
  let (columns, rows) = atlas_layout_dimensions;

  // Create a vertex buffer for all tiles
  for &tile in tiles {
    let sprite_index = tile
      .tile_type
      .calculate_sprite_index(&tile.terrain, &tile.climate, &resources);

    // Calculate vertices
    let base_idx = vertices.len() as u32;
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

    // Add indices for the tile
    indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);
  }

  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
  mesh.insert_indices(Indices::U32(indices));

  commands.entity(parent_entity).with_children(|parent| {
    parent.spawn((
      Mesh2d(meshes.add(mesh)),
      MeshMaterial2d(materials.add(ColorMaterial {
        alpha_mode: AlphaMode2d::Blend,
        texture: Some(texture),
        ..default()
      })),
      Transform::from_xyz(0.0, 0.0, layer),
      Name::new(format!("{:?} Tile Mesh", TerrainType::from(layer as usize))),
      TileMeshComponent {
        parent_entity,
        is_animated: false,
      },
    ));
  });
}

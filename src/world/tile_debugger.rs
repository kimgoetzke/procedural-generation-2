use crate::coords::Point;
use crate::events::{MouseClickEvent, RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::Settings;
use crate::world::components::{ChunkComponent, TileComponent};
use crate::world::resources::AssetPacks;
use crate::world::tile::Tile;
use crate::world::tile_type::get_sprite_index;
use bevy::app::{App, Plugin, Update};
use bevy::color::Color;
use bevy::core::Name;
use bevy::log::*;
use bevy::prelude::{
  default, Commands, Component, Entity, EventReader, JustifyText, OnAdd, OnRemove, Query, Res, ResMut, Resource, Text,
  Text2dBundle, TextStyle, Transform, Trigger, Vec3, Visibility, With,
};
use bevy::utils::{HashMap, HashSet};

pub struct TileDebuggerPlugin;

impl Plugin for TileDebuggerPlugin {
  fn build(&self, app: &mut App) {
    app
      .observe(on_add_chunk_component_trigger)
      .observe(on_remove_chunk_component_trigger)
      .observe(on_add_tile_component_trigger)
      .observe(on_left_mouse_click_trigger)
      .observe(on_remove_tile_component_trigger)
      .add_systems(Update, (toggle_tile_info_event, refresh_world_event))
      .init_resource::<TileEntityIndex>()
      .init_resource::<ChunkEntityIndex>();
  }
}

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource, Default)]
struct TileEntityIndex {
  pub grid: HashMap<Point, HashSet<Entity>>,
}

impl TileEntityIndex {
  pub fn get_entities(&self, point: Point) -> Vec<Entity> {
    let mut entities = Vec::new();
    if let Some(tiles) = self.grid.get(&point) {
      entities.extend(tiles.iter());
    }
    entities
  }
}

#[derive(Resource, Default)]
struct ChunkEntityIndex {
  pub grid: HashMap<Point, ChunkComponent>,
}

impl ChunkEntityIndex {
  pub fn get_entity(&self, point: Point) -> Option<ChunkComponent> {
    if let Some(entity) = self.grid.get(&point) {
      Some(entity.clone())
    } else {
      None
    }
  }
}

fn on_add_chunk_component_trigger(
  trigger: Trigger<OnAdd, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkEntityIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  index.grid.insert(cc.coords.world_grid, cc.clone());
}

fn on_remove_chunk_component_trigger(
  trigger: Trigger<OnRemove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkEntityIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  index.grid.remove(&cc.coords.world_grid);
}

fn on_add_tile_component_trigger(
  trigger: Trigger<OnAdd, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileEntityIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  index
    .grid
    .entry(tc.tile.coords.world_grid)
    .or_default()
    .insert(trigger.entity());
}

fn on_left_mouse_click_trigger(
  trigger: Trigger<MouseClickEvent>,
  tile_components: Query<&TileComponent>,
  chunk_component: Query<&ChunkComponent>,
  tile_index: Res<TileEntityIndex>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
  mut commands: Commands,
) {
  let event = trigger.event();
  let tile_component = tile_index
    .get_entities(event.coords.world_grid)
    .iter()
    .max_by_key(|e| tile_components.get(**e).unwrap().tile.layer)
    .map(|entity| {
      let tile_component = tile_components.get(*entity).unwrap();
      commands.spawn(tile_info(&asset_packs, &tile_component.tile, event.coords.world, &settings));
      tile_component
    });

  if let Some(tile_component) = tile_component {
    let parent_wg = tile_component.tile.get_parent_chunk_world_point();
    // TODO: Replace chunk_component with ChunkEntityIndex
    if let Some(parent_chunk) = chunk_component.iter().find(|cc| cc.coords.world == parent_wg) {
      debug!("Parent chunk at w{:?} contains the tiles listed below", parent_wg);
      for plane in &parent_chunk.layered_plane.planes {
        if let Some(tile) = plane.get_tile(tile_component.tile.coords.chunk_grid) {
          let neighbours = plane.get_neighbours(tile);
          neighbours.print(tile, neighbours.count_same());
        }
      }
    } else {
      warn!("Did not find parent chunk for tile at {:?}", tile_component.tile.coords);
    }
  }
}

fn on_remove_tile_component_trigger(
  trigger: Trigger<OnRemove, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileEntityIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  index.grid.entry(tc.tile.coords.world_grid).and_modify(|set| {
    set.remove(&trigger.entity());
  });
}

fn tile_info(
  asset_packs: &AssetPacks,
  tile: &Tile,
  spawn_point: Point,
  settings: &Res<Settings>,
) -> (Name, Text2dBundle, TileDebugInfoComponent) {
  let visibility = if settings.general.enable_tile_debugging {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
  (
    Name::new(format!("Tile wg{:?} Debug Info", tile.coords.world_grid)),
    Text2dBundle {
      text: Text::from_section(
        format!(
          "wg{:?} cg{:?}\n{:?}\n{:?}\nSprite index {:?}\nLayer {:?}",
          tile.coords.world_grid,
          tile.coords.chunk_grid,
          tile.terrain,
          tile.tile_type,
          get_sprite_index(&tile),
          tile.layer
        ),
        TextStyle {
          font: asset_packs.font.clone(),
          font_size: 22.,
          color: Color::WHITE,
        },
      )
      .with_justify(JustifyText::Center),
      visibility,
      transform: Transform {
        scale: Vec3::splat(0.1),
        translation: Vec3::new(spawn_point.x as f32, spawn_point.y as f32, tile.layer as f32 + 20.),
        ..Default::default()
      },
      ..default()
    },
    TileDebugInfoComponent,
  )
}

fn toggle_tile_info_event(
  mut events: EventReader<ToggleDebugInfo>,
  mut query: Query<&mut Visibility, With<TileDebugInfoComponent>>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for mut visibility in query.iter_mut() {
      *visibility = if settings.general.enable_tile_debugging {
        Visibility::Visible
      } else {
        Visibility::Hidden
      };
    }
  }
}

fn refresh_world_event(
  mut commands: Commands,
  mut events: EventReader<RefreshWorldEvent>,
  tile_debug_info: Query<Entity, With<TileDebugInfoComponent>>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for debug_info in tile_debug_info.iter() {
      commands.entity(debug_info).despawn();
    }
  }
}

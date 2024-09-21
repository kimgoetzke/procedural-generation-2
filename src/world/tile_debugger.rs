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
      .init_resource::<TileComponentIndex>()
      .init_resource::<ChunkComponentIndex>();
  }
}

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource, Default)]
struct TileComponentIndex {
  pub grid: HashMap<Point, HashSet<TileComponent>>,
}

impl TileComponentIndex {
  pub fn get_entities(&self, point: Point) -> Vec<&TileComponent> {
    let mut tile_components = Vec::new();
    if let Some(t) = self.grid.get(&point) {
      tile_components.extend(t.iter());
    }
    tile_components
  }
}

#[derive(Resource, Default)]
struct ChunkComponentIndex {
  pub grid: HashMap<Point, ChunkComponent>,
}

impl ChunkComponentIndex {
  pub fn get(&self, point: Point) -> Option<&ChunkComponent> {
    if let Some(entity) = self.grid.get(&point) {
      Some(entity)
    } else {
      None
    }
  }
}

fn on_add_chunk_component_trigger(
  trigger: Trigger<OnAdd, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  debug!("Adding chunk with key w{:?}", cc.coords.world);
  index.grid.insert(cc.coords.world, cc.clone());
}

fn on_remove_chunk_component_trigger(
  trigger: Trigger<OnRemove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  index.grid.remove(&cc.coords.world_grid);
}

fn on_add_tile_component_trigger(
  trigger: Trigger<OnAdd, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileComponentIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  index.grid.entry(tc.tile.coords.world_grid).or_default().insert(tc.clone());
}

fn on_left_mouse_click_trigger(
  trigger: Trigger<MouseClickEvent>,
  tile_index: Res<TileComponentIndex>,
  chunk_index: Res<ChunkComponentIndex>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
  mut commands: Commands,
) {
  if !settings.general.enable_tile_debugging {
    return;
  }
  let event = trigger.event();
  if let Some(tc) = tile_index
    .get_entities(event.coords.world_grid)
    .iter()
    .max_by_key(|tc| tc.tile.layer)
  {
    debug!("Debugging {:?}...", event.coords);
    commands.spawn(tile_info(&asset_packs, &tc.tile, event.coords.world, &settings));
    let parent_wg = tc.tile.get_parent_chunk_world_point();
    if let Some(parent_chunk) = chunk_index.get(parent_wg) {
      debug!("Parent is chunk w{:?}; any tiles are listed below", parent_wg);
      for plane in &parent_chunk.layered_plane.planes {
        if let Some(tile) = plane.get_tile(tc.tile.coords.chunk_grid) {
          let neighbours = plane.get_neighbours(tile);
          neighbours.print(tile, neighbours.count_same());
        }
      }
    } else {
      warn!(
        "Failed to find parent chunk at w{} for tile at {:?}",
        parent_wg, tc.tile.coords
      );
    }
  }
}

fn on_remove_tile_component_trigger(
  trigger: Trigger<OnRemove, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileComponentIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  index.grid.entry(tc.tile.coords.world_grid).and_modify(|set| {
    set.remove(&tc.clone());
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

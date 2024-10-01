use crate::constants::TILE_SIZE;
use crate::coords::Point;
use crate::events::{MouseClickEvent, RegenerateWorldEvent, ToggleDebugInfo};
use crate::generation::components::TileComponent;
use crate::generation::resources::{AssetPacks, ChunkComponentIndex};
use crate::generation::tile::Tile;
use crate::generation::tile_type::get_sprite_index;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::log::*;
use bevy::prelude::{
  default, Commands, Component, Entity, EventReader, JustifyText, OnAdd, OnRemove, Query, Res, ResMut, Resource, Text,
  Text2dBundle, TextStyle, Transform, Trigger, Vec3, Visibility, With,
};
use bevy::sprite::Anchor;
use bevy::utils::{HashMap, HashSet};

pub struct TileDebuggerPlugin;

impl Plugin for TileDebuggerPlugin {
  fn build(&self, app: &mut App) {
    app
      .observe(on_add_tile_component_trigger)
      .observe(on_left_mouse_click_trigger)
      .observe(on_remove_tile_component_trigger)
      .add_systems(Update, (toggle_tile_info_event, regenerate_world_event))
      .init_resource::<TileComponentIndex>();
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
    .get_entities(event.world_grid)
    .iter()
    .max_by_key(|tc| tc.tile.layer)
  {
    // TODO: Fix this again - it's not detecting the debuggable tile coords correctly
    debug!("You are debugging w{:?} wg{:?}", event.world, event.world_grid);
    commands.spawn(tile_info(&asset_packs, &tc.tile, event.world, &settings));
    let parent_w = tc.tile.get_parent_chunk_world();
    if let Some(parent_chunk) = chunk_index.get(parent_w) {
      debug!("Parent is chunk w{:?}; any tiles are listed below", parent_w);
      for plane in &parent_chunk.layered_plane.planes {
        if let Some(tile) = plane.get_tile(tc.tile.coords.chunk_grid) {
          let neighbours = plane.get_neighbours(tile);
          neighbours.log(tile, neighbours.count_same());
        }
      }
    } else {
      error!(
        "Failed to find parent chunk at w{} for tile at {:?}",
        parent_w, tc.tile.coords
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
  _asset_packs: &AssetPacks,
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
      text_anchor: Anchor::Center,
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
          font_size: 30.,
          ..default()
        },
      )
      .with_justify(JustifyText::Center),
      visibility,
      transform: Transform {
        scale: Vec3::splat(0.1),
        translation: Vec3::new(
          spawn_point.x as f32 + TILE_SIZE as f32 / 2.,
          spawn_point.y as f32 - TILE_SIZE as f32 / 2.,
          tile.layer as f32 + 1000.,
        ),
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

fn regenerate_world_event(
  mut commands: Commands,
  mut events: EventReader<RegenerateWorldEvent>,
  tile_debug_info: Query<Entity, With<TileDebugInfoComponent>>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for debug_info in tile_debug_info.iter() {
      commands.entity(debug_info).despawn();
    }
  }
}

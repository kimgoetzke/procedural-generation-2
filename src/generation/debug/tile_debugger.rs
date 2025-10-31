use crate::constants::*;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::generation::lib::{GenerationResourcesCollection, ObjectComponent, Tile, TileMeshComponent};
use crate::generation::resources::ChunkComponentIndex;
use crate::messages::{MouseRightClickMessage, RegenerateWorldMessage, ToggleDebugInfoMessage};
use crate::resources::Settings;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::log::*;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{
  Add, Commands, Component, Entity, IntoSystem, Justify, MessageReader, Name, Observer, On, Query, Remove, Res, ResMut,
  Resource, Text2d, TextFont, Transform, Vec3, Visibility, With, default,
};
use bevy::sprite::Anchor;
use bevy::text::{LineBreak, TextBounds, TextColor, TextLayout};

pub struct TileDebuggerPlugin;

impl Plugin for TileDebuggerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, spawn_observers_system)
      .add_systems(
        Update,
        (
          toggle_tile_info_message,
          regenerate_world_message,
          on_right_mouse_click_message,
        ),
      )
      .init_resource::<TileMeshComponentIndex>()
      .init_resource::<ObjectComponentIndex>();
  }
}

const MARGIN: f32 = 2.;

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource, Default)]
struct TileMeshComponentIndex {
  map: HashMap<Point<ChunkGrid>, HashSet<TileMeshComponent>>,
}

impl TileMeshComponentIndex {
  pub fn get_entities(&self, cg: Point<ChunkGrid>, tg: Point<TileGrid>) -> Vec<&Tile> {
    let mut tiles: Vec<&Tile> = Vec::new();
    if let Some(tile_mesh_component_set) = self.map.get(&cg) {
      tile_mesh_component_set.iter().for_each(|m| {
        m.find_all(&tg).iter().for_each(|t| {
          if t.coords.tile_grid == tg {
            tiles.push(t);
          }
        })
      });
    }

    tiles
  }
}

#[derive(Resource, Default)]
struct ObjectComponentIndex {
  map: HashMap<Point<TileGrid>, ObjectComponent>,
}

impl ObjectComponentIndex {
  pub fn get(&self, point: Point<TileGrid>) -> Option<&ObjectComponent> {
    if let Some(t) = self.map.get(&point) { Some(t) } else { None }
  }
}

fn spawn_observers_system(world: &mut bevy::ecs::world::World) {
  world.spawn_batch([
    (
      Observer::new(IntoSystem::into_system(on_add_object_component_trigger)),
      Name::new("Observer: Add ObjectComponent"),
    ),
    (
      Observer::new(IntoSystem::into_system(on_remove_object_component_trigger)),
      Name::new("Observer: Remove ObjectComponent"),
    ),
    (
      Observer::new(IntoSystem::into_system(on_add_tile_mesh_component_trigger)),
      Name::new("Observer: Add TileMeshComponent"),
    ),
    (
      Observer::new(IntoSystem::into_system(on_remove_tile_mesh_component_trigger)),
      Name::new("Observer: Remove TileMeshComponent"),
    ),
  ]);
}

fn on_add_object_component_trigger(
  trigger: On<Add, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.event().entity).expect("Failed to get ObjectComponent");
  index.map.insert(oc.coords.tile_grid, oc.clone());
}

fn on_remove_object_component_trigger(
  trigger: On<Remove, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.event().entity).expect("Failed to get ObjectComponent");
  index.map.remove(&oc.coords.tile_grid);
}

fn on_add_tile_mesh_component_trigger(
  trigger: On<Add, TileMeshComponent>,
  query: Query<&TileMeshComponent>,
  mut index: ResMut<TileMeshComponentIndex>,
) {
  let tmc = query.get(trigger.event().entity).expect("Failed to get TileMeshComponent");
  index.map.entry(tmc.cg()).or_default().insert(tmc.clone());
}

fn on_remove_tile_mesh_component_trigger(
  trigger: On<Remove, TileMeshComponent>,
  query: Query<&TileMeshComponent>,
  mut index: ResMut<TileMeshComponentIndex>,
) {
  let tmc = query.get(trigger.event().entity).expect("Failed to get TileMeshComponent");
  index.map.entry(tmc.cg()).and_modify(|set| {
    set.remove(&tmc.clone());
  });
}

fn on_right_mouse_click_message(
  mut right_click_messages: MessageReader<MouseRightClickMessage>,
  object_index: Res<ObjectComponentIndex>,
  tile_index: Res<TileMeshComponentIndex>,
  chunk_index: Res<ChunkComponentIndex>,
  resources: Res<GenerationResourcesCollection>,
  settings: Res<Settings>,
  mut commands: Commands,
) {
  if !settings.general.enable_tile_debugging {
    return;
  }
  for message in right_click_messages.read() {
    if let Some(tile) = tile_index.get_entities(message.cg, message.tg).iter().max_by_key(|t| t.layer) {
      debug!("You are debugging {} {} {}", message.tile_w, message.cg, message.tg);
      let object_component = object_index.get(message.tg);
      commands.spawn(tile_info(&resources, &tile, message.tile_w, &settings, &object_component));
      let parent_w = tile.get_parent_chunk_w();
      if let Some(parent_chunk) = chunk_index.get(&parent_w) {
        debug!("Parent of {} is chunk {}/{}", message.tg, parent_w, message.cg);
        for plane in &parent_chunk.layered_plane.planes {
          if let Some(tile) = plane.get_tile(tile.coords.internal_grid) {
            let neighbours = plane.get_neighbours(tile);
            neighbours.log(tile, neighbours.count_same());
          }
        }
        debug!("{:?}", tile.debug_data);
      } else {
        error!("Failed to find parent chunk at {} for tile at {:?}", parent_w, tile.coords);
      }
      if let Some(oc) = object_index.get(message.tg) {
        debug!("{:?}", oc);
      } else {
        debug!(
          "No object(s) found at {:?} {:?} which is inside {}",
          message.tile_w, message.tg, message.cg
        );
      }
    }
  }
}

fn tile_info(
  resources: &GenerationResourcesCollection,
  tile: &Tile,
  spawn_point: Point<World>,
  settings: &Res<Settings>,
  object_component_option: &Option<&ObjectComponent>,
) -> (
  Name,
  Anchor,
  Text2d,
  TextFont,
  TextLayout,
  TextBounds,
  TextColor,
  Visibility,
  Transform,
  TileDebugInfoComponent,
) {
  let object = if let Some(oc) = object_component_option {
    format!(
      "\nObject: \n{:?}\n(Sprite {}, layer {})",
      oc.object_name, oc.sprite_index, oc.layer
    )
  } else {
    "\nNo object sprite".to_string()
  };
  let visibility = if settings.general.enable_tile_debugging {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
  let sprite_index = tile.tile_type.calculate_sprite_index(&tile.terrain, &tile.climate, resources);
  (
    Name::new(format!("Tile {:?} Debug Info", tile.coords.tile_grid)),
    Anchor::TOP_LEFT,
    Text2d::new(format!(
      "{}\n{} {}\n{:?}\n{:?}\n(Sprite {:?}, layer {:?})\n{}",
      tile.coords.chunk_grid,
      tile.coords.tile_grid,
      tile.coords.internal_grid,
      tile.terrain,
      tile.tile_type,
      sprite_index,
      tile.layer,
      object
    )),
    TextFont {
      font_size: 22.,
      ..default()
    },
    TextLayout::new(Justify::Left, LineBreak::AnyCharacter),
    TextBounds::new((TILE_SIZE as f32 - MARGIN) * 10., (TILE_SIZE as f32 - MARGIN) * 10.),
    TextColor(LIGHT),
    visibility,
    Transform {
      scale: Vec3::splat(0.1),
      translation: Vec3::new(
        spawn_point.x as f32 + (MARGIN / 2.),
        spawn_point.y as f32 - (MARGIN / 2.),
        tile.layer as f32 + 20000.,
      ),
      ..Default::default()
    },
    TileDebugInfoComponent,
  )
}

fn toggle_tile_info_message(
  mut messages: MessageReader<ToggleDebugInfoMessage>,
  mut query: Query<&mut Visibility, With<TileDebugInfoComponent>>,
  settings: Res<Settings>,
) {
  let message_count = messages.read().count();
  if message_count > 0 {
    for mut visibility in query.iter_mut() {
      *visibility = if settings.general.enable_tile_debugging {
        Visibility::Visible
      } else {
        Visibility::Hidden
      };
    }
  }
}

fn regenerate_world_message(
  mut commands: Commands,
  mut messages: MessageReader<RegenerateWorldMessage>,
  tile_debug_info: Query<Entity, With<TileDebugInfoComponent>>,
) {
  let message_count = messages.read().count();
  if message_count > 0 {
    for debug_info in tile_debug_info.iter() {
      commands.entity(debug_info).despawn();
    }
  }
}

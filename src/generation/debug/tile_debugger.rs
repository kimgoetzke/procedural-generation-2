use crate::constants::*;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::events::{MouseClickEvent, RegenerateWorldEvent, ToggleDebugInfo};
use crate::generation::lib::{GenerationResourcesCollection, ObjectComponent, Tile, TileMeshComponent};
use crate::generation::resources::ChunkComponentIndex;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::log::*;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{
  Commands, Component, Entity, EventReader, IntoSystem, JustifyText, Name, Observer, OnAdd, OnRemove, Query, Res, ResMut,
  Resource, Text2d, TextFont, Transform, Trigger, Vec3, Visibility, With, default,
};
use bevy::sprite::Anchor;
use bevy::text::{LineBreak, TextBounds, TextColor, TextLayout};

pub struct TileDebuggerPlugin;

impl Plugin for TileDebuggerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, spawn_observers_system)
      .add_systems(Update, (toggle_tile_info_event, regenerate_world_event))
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
    (
      Observer::new(IntoSystem::into_system(on_left_mouse_click_trigger)),
      Name::new("Observer: MouseClickEvent"),
    ),
  ]);
}

fn on_add_object_component_trigger(
  trigger: Trigger<OnAdd, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.target()).expect("Failed to get ObjectComponent");
  index.map.insert(oc.coords.tile_grid, oc.clone());
}

fn on_remove_object_component_trigger(
  trigger: Trigger<OnRemove, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.target()).expect("Failed to get ObjectComponent");
  index.map.remove(&oc.coords.tile_grid);
}

fn on_add_tile_mesh_component_trigger(
  trigger: Trigger<OnAdd, TileMeshComponent>,
  query: Query<&TileMeshComponent>,
  mut index: ResMut<TileMeshComponentIndex>,
) {
  let tmc = query.get(trigger.target()).expect("Failed to get TileMeshComponent");
  index.map.entry(tmc.cg()).or_default().insert(tmc.clone());
}

fn on_remove_tile_mesh_component_trigger(
  trigger: Trigger<OnRemove, TileMeshComponent>,
  query: Query<&TileMeshComponent>,
  mut index: ResMut<TileMeshComponentIndex>,
) {
  let tmc = query.get(trigger.target()).expect("Failed to get TileMeshComponent");
  index.map.entry(tmc.cg()).and_modify(|set| {
    set.remove(&tmc.clone());
  });
}

fn on_left_mouse_click_trigger(
  trigger: Trigger<MouseClickEvent>,
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
  let event = trigger.event();
  if let Some(tile) = tile_index.get_entities(event.cg, event.tg).iter().max_by_key(|t| t.layer) {
    debug!("You are debugging {} {} {}", event.tile_w, event.cg, event.tg);
    let object_component = object_index.get(event.tg);
    commands.spawn(tile_info(&resources, &tile, event.tile_w, &settings, &object_component));
    let parent_w = tile.get_parent_chunk_w();
    if let Some(parent_chunk) = chunk_index.get(&parent_w) {
      debug!("Parent of {} is chunk {}/{}", event.tg, parent_w, event.cg);
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
    if let Some(oc) = object_index.get(event.tg) {
      debug!("{:?}", oc);
    } else {
      debug!(
        "No object(s) found at {:?} {:?} which is inside {}",
        event.tile_w, event.tg, event.cg
      );
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
    Anchor::TopLeft,
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
    TextLayout::new(JustifyText::Left, LineBreak::AnyCharacter),
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

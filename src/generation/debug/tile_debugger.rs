use crate::constants::*;
use crate::coords::point::{TileGrid, World};
use crate::coords::Point;
use crate::events::{MouseClickEvent, RegenerateWorldEvent, ToggleDebugInfo};
use crate::generation::lib::{ObjectComponent, Tile, TileComponent};
use crate::generation::resources::{ChunkComponentIndex, GenerationResourcesCollection};
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::log::*;
use bevy::prelude::{
  default, Commands, Component, Entity, EventReader, JustifyText, OnAdd, OnRemove, Query, Res, ResMut, Resource, Text2d,
  TextFont, Transform, Trigger, Vec3, Visibility, With,
};
use bevy::sprite::Anchor;
use bevy::text::{LineBreak, TextBounds, TextColor, TextLayout};
use bevy::utils::{HashMap, HashSet};

pub struct TileDebuggerPlugin;

impl Plugin for TileDebuggerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_observer(on_add_object_component_trigger)
      .add_observer(on_add_tile_component_trigger)
      .add_observer(on_left_mouse_click_trigger)
      .add_observer(on_remove_tile_component_trigger)
      .add_observer(on_remove_object_component_trigger)
      .add_systems(Update, (toggle_tile_info_event, regenerate_world_event))
      .init_resource::<TileComponentIndex>()
      .init_resource::<ObjectComponentIndex>();
  }
}

const MARGIN: f32 = 2.;

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource, Default)]
struct TileComponentIndex {
  map: HashMap<Point<TileGrid>, HashSet<TileComponent>>,
}

impl TileComponentIndex {
  pub fn get_entities(&self, point: Point<TileGrid>) -> Vec<&TileComponent> {
    let mut tile_components = Vec::new();
    if let Some(t) = self.map.get(&point) {
      tile_components.extend(t.iter());
    }
    tile_components
  }
}

#[derive(Resource, Default)]
struct ObjectComponentIndex {
  map: HashMap<Point<TileGrid>, ObjectComponent>,
}

impl ObjectComponentIndex {
  pub fn get(&self, point: Point<TileGrid>) -> Option<&ObjectComponent> {
    if let Some(t) = self.map.get(&point) {
      Some(t)
    } else {
      None
    }
  }
}

fn on_add_object_component_trigger(
  trigger: Trigger<OnAdd, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.entity()).expect("Failed to get ObjectComponent");
  index.map.insert(oc.coords.tile_grid, oc.clone());
}

fn on_remove_object_component_trigger(
  trigger: Trigger<OnRemove, ObjectComponent>,
  query: Query<&ObjectComponent>,
  mut index: ResMut<ObjectComponentIndex>,
) {
  let oc = query.get(trigger.entity()).expect("Failed to get ObjectComponent");
  index.map.remove(&oc.coords.tile_grid);
}

fn on_add_tile_component_trigger(
  trigger: Trigger<OnAdd, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileComponentIndex>,
) {
  let tc = query.get(trigger.entity()).expect("Failed to get TileComponent");
  index.map.entry(tc.tile.coords.tile_grid).or_default().insert(tc.clone());
}

fn on_remove_tile_component_trigger(
  trigger: Trigger<OnRemove, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<TileComponentIndex>,
) {
  let tc = query.get(trigger.entity()).expect("Failed to get TileComponent");
  index.map.entry(tc.tile.coords.tile_grid).and_modify(|set| {
    set.remove(&tc.clone());
  });
}

fn on_left_mouse_click_trigger(
  trigger: Trigger<MouseClickEvent>,
  object_index: Res<ObjectComponentIndex>,
  tile_index: Res<TileComponentIndex>,
  chunk_index: Res<ChunkComponentIndex>,
  resources: Res<GenerationResourcesCollection>,
  settings: Res<Settings>,
  mut commands: Commands,
) {
  if !settings.general.enable_tile_debugging {
    return;
  }
  let event = trigger.event();
  if let Some(tc) = tile_index.get_entities(event.tg).iter().max_by_key(|tc| tc.tile.layer) {
    debug!("You are debugging {} {} {}", event.tile_w, event.cg, event.tg);
    let object_component = object_index.get(event.tg);
    commands.spawn(tile_info(&resources, &tc.tile, event.tile_w, &settings, &object_component));
    let parent_w = tc.tile.get_parent_chunk_w();
    if let Some(parent_chunk) = chunk_index.get(parent_w) {
      debug!("Parent of {} is chunk {}/{}", event.tg, parent_w, event.cg);
      for plane in &parent_chunk.layered_plane.planes {
        if let Some(tile) = plane.get_tile(tc.tile.coords.internal_grid) {
          let neighbours = plane.get_neighbours(tile);
          neighbours.log(tile, neighbours.count_same());
        }
      }
      debug!("{:?}", tc.tile.debug_data);
    } else {
      error!("Failed to find parent chunk at {} for tile at {:?}", parent_w, tc.tile.coords);
    }
    if let Some(oc) = object_index.get(event.tg) {
      debug!("{:?}", oc);
    } else {
      debug!("No object(s) found at {:?} {:?}", event.tile_w, event.tg);
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
    format!("\nObject: \n{:?}\n(Sprite {})", oc.object_name, oc.sprite_index)
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
        tile.layer as f32 + 1000.,
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

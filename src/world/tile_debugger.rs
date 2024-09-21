use crate::coords::Point;
use crate::events::{MouseClickEvent, RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::Settings;
use crate::world::components::TileComponent;
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
      .observe(on_add_tile_component_trigger)
      .observe(on_left_mouse_click_trigger)
      .observe(on_remove_tile_component_trigger)
      .add_systems(Update, (toggle_tile_info_event, refresh_world_event))
      .init_resource::<SpatialTileEntityIndex>();
  }
}

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource, Default)]
struct SpatialTileEntityIndex {
  pub grid: HashMap<(i32, i32), HashSet<Entity>>,
}

impl SpatialTileEntityIndex {
  pub fn get_entities_for_location(&self, point: Point) -> Vec<Entity> {
    let mut entities = Vec::new();
    if let Some(tiles) = self.grid.get(&(point.x, point.y)) {
      entities.extend(tiles.iter());
    }
    entities
  }

  pub fn get_nearby_entities(&self, point: Point) -> Vec<Entity> {
    let mut nearby = Vec::new();
    if let Some(tiles) = self.grid.get(&(point.x, point.y)) {
      nearby.extend(tiles.iter());
    }
    // for x in -1..2 {
    //   for y in -1..2 {
    //     if let Some(tiles) = self.grid.get(&(point.x + x, point.y + y)) {
    //       nearby.extend(tiles.iter());
    //     }
    //   }
    // }
    nearby
  }
}

fn on_add_tile_component_trigger(
  trigger: Trigger<OnAdd, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<SpatialTileEntityIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  let key = (tc.tile.coords.tile.x, tc.tile.coords.tile.y);
  index.grid.entry(key).or_default().insert(trigger.entity());
}

fn on_left_mouse_click_trigger(
  trigger: Trigger<MouseClickEvent>,
  tile_components: Query<&TileComponent>,
  index: Res<SpatialTileEntityIndex>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
  mut commands: Commands,
) {
  let event = trigger.event();
  let relevant_entities = index.get_entities_for_location(event.coords.tile);

  if let Some(entity) = relevant_entities.iter().max_by_key(|e| {
    let tc = tile_components.get(**e).unwrap();
    tc.tile.layer
  }) {
    let tile_component = tile_components.get(*entity).unwrap();
    commands.spawn(tile_info(&asset_packs, &tile_component.tile, event.coords.world, &settings));
  }

  for entity in relevant_entities {
    let tile_component = tile_components.get(entity).unwrap();
    debug!("Found: {:?}", tile_component);
  }
}

fn on_remove_tile_component_trigger(
  trigger: Trigger<OnRemove, TileComponent>,
  query: Query<&TileComponent>,
  mut index: ResMut<SpatialTileEntityIndex>,
) {
  let tc = query.get(trigger.entity()).unwrap();
  let key = (tc.tile.coords.tile.x, tc.tile.coords.tile.y);
  index.grid.entry(key).and_modify(|set| {
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
    Name::new(format!("Tile t{:?} Debug Info", tile.coords.tile)),
    Text2dBundle {
      text: Text::from_section(
        format!(
          "t{:?} c{:?}\n{:?}\n{:?}\nSprite index {:?}\nLayer {:?}",
          tile.coords.tile,
          tile.coords.chunk,
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

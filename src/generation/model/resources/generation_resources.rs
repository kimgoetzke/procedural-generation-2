use crate::generation::model::resources::object_resources::ObjectResources;
use crate::generation::model::resources::settlement_resources::SettlementResources;
use crate::generation::model::resources::world_resources::WorldResources;
use bevy::prelude::Resource;

/// A collection of all assets, rules, and other data that is used when spawning terrain and object sprites in the
/// world. Initialised in the [`crate::generation::generation_resources::GenerationResourcesPlugin`] on startup.
///
/// It also stores object assets, wave function collapse terrain states, and resolved settlement templates used during
/// object generation.
#[derive(Resource, Default, Debug, Clone)]
pub struct GenerationResources {
  pub world: WorldResources,
  pub objects: ObjectResources,
  pub settlements: SettlementResources,
}

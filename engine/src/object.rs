use std::sync::Arc;

use gusto_core::{ObjectDefinition, ObjectManifest};
use tokio::sync::RwLock;
use uuid::Uuid;

pub type ObjectId = Uuid;

/// Object
pub struct Object<O>
where
  O: ObjectDefinition,
{
  pub id: ObjectId,
  pub manifest: ObjectManifest<O>,
  pub state: Arc<RwLock<O::State>>,
}

impl<O> Object<O>
where
  O: ObjectDefinition,
{
  pub fn new(manifest: ObjectManifest<O>, state: O::State) -> Self {
    Self {
      id: Uuid::new_v4(),
      manifest,
      state: Arc::new(RwLock::new(state)),
    }
  }
}

impl<O> Clone for Object<O>
where
  O: ObjectDefinition,
{
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      manifest: self.manifest.clone(),
      state: self.state.clone(),
    }
  }
}

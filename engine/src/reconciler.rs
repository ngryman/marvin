use std::{collections::HashSet, marker::PhantomData, sync::Arc};

use gusto_core::{Controller, ObjectDefinition};
use parking_lot::RwLock;

use crate::Object;

/// Reconciler
pub struct Reconciler<C, O> {
  pending: Arc<RwLock<HashSet<String>>>,
  c: PhantomData<C>,
  o: PhantomData<O>,
}

impl<C, O> Reconciler<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  pub fn reconcile(
    &mut self,
    name: String,
    object: Object<O>,
    controller: Arc<C>,
  ) {
    if self.pending.read().contains(&name) {
      println!("skip reconciliation");
      return;
    }

    let pending = self.pending.clone();

    tokio::spawn(async move {
      let manifest = &object.manifest;
      let state = &mut object.state.write().await;

      if let Err(e) = controller.reconcile(manifest, state).await {
        controller.reconcile_error(e).await;
      }

      pending.write().remove(&manifest.meta.name);
    });

    self.pending.write().insert(name);
  }
}

impl<C, O> Default for Reconciler<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  fn default() -> Self {
    Self {
      pending: Default::default(),
      c: Default::default(),
      o: Default::default(),
    }
  }
}

use std::{collections::HashSet, sync::Arc};

use gusto_core::{Command, Controller, ObjectDefinition};
use parking_lot::RwLock;

use crate::{Object, ObjectId};

/// Reconciler
pub struct Reconciler<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  pending: Arc<RwLock<HashSet<ObjectId>>>,
  command: Command<O>,
  controller: Arc<C>,
}

impl<C, O> Reconciler<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  pub fn new(command: Command<O>, controller: Arc<C>) -> Self {
    Self {
      pending: Default::default(),
      command,
      controller,
    }
  }

  pub fn reconcile(&mut self, object: Object<O>) {
    if self.pending.read().contains(&object.id) {
      println!("skip reconciliation");
      return;
    }

    let pending = self.pending.clone();
    let command = self.command.clone();
    let controller = self.controller.clone();

    tokio::spawn(async move {
      let manifest = &object.manifest;
      let state = &mut object.state.write().await;

      if let Err(e) = controller.reconcile(manifest, state, &command).await {
        controller.reconcile_error(e).await;
      }

      pending.write().remove(&object.id);
    });

    self.pending.write().insert(object.id);
  }
}

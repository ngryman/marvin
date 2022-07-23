use std::{collections::BTreeMap, sync::Arc};

use anyhow::{bail, Result};
use gusto_core::{Command, Controller, ObjectDefinition, ObjectManifest};

use crate::{
  store::{Change, StoreEvent}, Object, Reconciler, Store
};

/// Objects
struct Objects<O>
where
  O: ObjectDefinition,
{
  inner: BTreeMap<String, Object<O>>,
}

impl<O> Objects<O>
where
  O: ObjectDefinition,
{
  pub fn insert(&mut self, name: String, object: Object<O>) {
    self.inner.insert(name, object);
  }

  pub fn patch_manifest(
    &mut self,
    name: &str,
    manifest: ObjectManifest<O>,
  ) -> Option<Object<O>> {
    if let Some(object) = self.inner.get_mut(name) {
      object.manifest = manifest;
      Some(object.clone())
    } else {
      None
    }
  }

  pub fn remove(&mut self, name: &str) -> Option<Object<O>> {
    self.inner.remove(name)
  }
}

impl<O> Default for Objects<O>
where
  O: ObjectDefinition,
{
  fn default() -> Self {
    Self {
      inner: Default::default(),
    }
  }
}

/// Operator
pub struct Operator<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  controller: Arc<C>,
  reconciler: Reconciler<C, O>,
  objects: Objects<O>,
  store: Arc<Store<O>>,
}

impl<C, O> Operator<C, O>
where
  C: Controller<O>,
  O: ObjectDefinition,
{
  pub fn new(controller: C, command: Command<O>, store: Arc<Store<O>>) -> Self {
    let controller = Arc::new(controller);

    Self {
      controller: controller.clone(),
      reconciler: Reconciler::new(command, controller),
      objects: Default::default(),
      store,
    }
  }

  pub async fn start(&mut self) {
    let events_rx = self.store.events();

    while let Ok(event) = events_rx.recv_async().await {
      println!("received store event: {:?}", event.change);

      if let Err(e) = self.handle_event(event).await {
        eprintln!("{e}");
      }
    }
  }

  async fn handle_event(&mut self, event: StoreEvent<O>) -> Result<()> {
    let StoreEvent { change, manifest } = event;
    let name = manifest.meta.name.clone();

    match change {
      Change::Create => {
        println!("{}: admit manifest", name);
        let manifest = self.controller.admit_manifest(manifest).await?;
        self.store.patch(manifest.clone());

        println!("{}: initialize state", name);
        let state = self.controller.initialize_state(&manifest).await?;

        let object = Object::new(manifest.clone(), state);
        self.objects.insert(name.clone(), object.clone());

        if self.controller.should_reconcile(&manifest).await? {
          self.reconciler.reconcile(name, object);
        }
      }
      Change::Update => {
        if let Some(object) =
          self.objects.patch_manifest(&name, manifest.clone())
        {
          println!("{}: admit manifest", name);
          let manifest = self.controller.admit_manifest(manifest).await?;
          self.store.patch(manifest.clone());

          if self.controller.should_reconcile(&manifest).await? {
            self.reconciler.reconcile(name, object);
          }
        } else {
          bail!("no object found for name '{}'", name)
        }
      }
      Change::Delete => {
        if self.objects.remove(&name).is_some() {
          println!("{}: terminate", name);
          self.controller.terminate(&manifest).await?;

          self.objects.remove(&name);
        } else {
          bail!("no object found for name '{}'", name)
        }
      }
    }

    Ok(())
  }
}

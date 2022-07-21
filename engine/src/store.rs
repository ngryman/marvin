use std::{collections::BTreeMap, sync::Arc};

use anyhow::{anyhow, Result};
use gusto_core::{ObjectDefinition, ObjectManifest};
use parking_lot::RwLock;
use tokio::sync::broadcast::{channel, Receiver, Sender};

/// StoreEvent
pub struct StoreEvent<O>
where
  O: ObjectDefinition,
{
  pub change: Change,
  pub manifest: ObjectManifest<O>,
}

impl<O> StoreEvent<O>
where
  O: ObjectDefinition,
{
  pub fn new(change: Change, manifest: ObjectManifest<O>) -> Self {
    Self { change, manifest }
  }
}

impl<O> Clone for StoreEvent<O>
where
  O: ObjectDefinition,
{
  fn clone(&self) -> Self {
    Self {
      change: self.change.clone(),
      manifest: self.manifest.clone(),
    }
  }
}

/// Change
#[derive(Clone, Debug)]
pub enum Change {
  Create,
  Update,
  Delete,
}

pub struct Store<O>
where
  O: ObjectDefinition,
{
  manifests: Arc<RwLock<BTreeMap<String, ObjectManifest<O>>>>,
  event_tx: Sender<StoreEvent<O>>,
}

impl<O> Store<O>
where
  O: ObjectDefinition,
{
  pub fn insert(&mut self, manifest: ObjectManifest<O>) -> Result<()> {
    let prev = self
      .manifests
      .write()
      .insert(manifest.meta.name.clone(), manifest.clone());

    match prev {
      Some(_) => {
        self
          .event_tx
          .send(StoreEvent::new(Change::Update, manifest))
      }
      None => {
        self
          .event_tx
          .send(StoreEvent::new(Change::Create, manifest))
      }
    }
    .map_err(|_| anyhow!("closed channel"))?;

    Ok(())
  }

  pub fn patch(&self, manifest: ObjectManifest<O>) {
    self
      .manifests
      .write()
      .insert(manifest.meta.name.clone(), manifest);
  }

  pub fn remove(&mut self, name: &str) -> Result<()> {
    if let Some(removed) = self.manifests.write().remove(name) {
      self
        .event_tx
        .send(StoreEvent::new(Change::Delete, removed))
        .map_err(|_| anyhow!("closed channel"))?;
    }

    Ok(())
  }

  pub fn events(&self) -> Receiver<StoreEvent<O>> {
    self.event_tx.subscribe()
  }
}

impl<O> Default for Store<O>
where
  O: ObjectDefinition,
{
  fn default() -> Self {
    let (event_tx, _) = channel::<StoreEvent<O>>(1);
    Self {
      manifests: Default::default(),
      event_tx,
    }
  }
}

impl<O> Clone for Store<O>
where
  O: ObjectDefinition,
{
  fn clone(&self) -> Self {
    Self {
      manifests: self.manifests.clone(),
      event_tx: self.event_tx.clone(),
    }
  }
}

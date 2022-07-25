use std::{any::Any, collections::BTreeMap, sync::Arc};

use anyhow::{anyhow, bail, Result};
use flume::{Receiver, Sender};
use gusto_core::{
  util::Safe, DynObjectManifest, ObjectDefinition, ObjectManifest
};
use parking_lot::RwLock;

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
  manifests: RwLock<BTreeMap<String, ObjectManifest<O>>>,
  event_tx: Sender<StoreEvent<O>>,
  event_rx: Receiver<StoreEvent<O>>,
}

impl<O> Store<O>
where
  O: ObjectDefinition,
{
  pub fn insert(&self, manifest: ObjectManifest<O>) -> Result<()> {
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

  pub fn patch(&self, manifest: ObjectManifest<O>) -> Result<()> {
    let name = &manifest.meta.name;

    if let Some(existing) = self.manifests.write().get_mut(name) {
      *existing = manifest;
    } else {
      bail!("cannot patch, not manifest found with name {name}")
    }

    Ok(())
  }

  pub fn remove(&self, name: &str) -> Result<()> {
    if let Some(removed) = self.manifests.write().remove(name) {
      self
        .event_tx
        .send(StoreEvent::new(Change::Delete, removed))
        .map_err(|_| anyhow!("closed channel"))?;
    }

    Ok(())
  }

  pub fn events(&self) -> Receiver<StoreEvent<O>> {
    self.event_rx.clone()
  }
}

impl<O> Default for Store<O>
where
  O: ObjectDefinition,
{
  fn default() -> Self {
    let (event_tx, event_rx) = flume::unbounded::<StoreEvent<O>>();
    Self {
      manifests: Default::default(),
      event_tx,
      event_rx,
    }
  }
}

/// AnyStore
pub trait AnyStore: Any + Safe {
  fn insert(&self, manifest: Box<DynObjectManifest>) -> Result<()>;
  fn remove(&self, name: &str) -> Result<()>;
}

impl<O> AnyStore for Store<O>
where
  O: ObjectDefinition,
{
  fn insert(&self, manifest: Box<DynObjectManifest>) -> Result<()> {
    let manifest = Box::into_inner(manifest.as_manifest()?);
    Store::<O>::insert(self, manifest)
  }

  fn remove(&self, name: &str) -> Result<()> {
    Store::<O>::remove(self, name)
  }
}

impl dyn AnyStore + Send + Sync {
  pub fn as_store<O>(self: Arc<Self>) -> Result<Arc<Store<O>>>
  where
    O: ObjectDefinition,
  {
    Arc::downcast(self)
      .map_err(|_| anyhow!("cannot downcast to {}", std::any::type_name::<O>()))
  }
}

/// DynStore
pub type DynStore = dyn AnyStore + Send + Sync;

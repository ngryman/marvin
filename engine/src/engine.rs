use std::{
  collections::{BTreeMap, VecDeque}, sync::Arc
};

use anyhow::{anyhow, Result};
use flume::{Receiver, Sender};
use gusto_core::{Command, CommandEvent, Controller, ObjectDefinition};
use tokio::task::JoinHandle;

use crate::{DynStore, Operator, Owners, Store};

type StartOperatorFn = Box<dyn FnOnce() -> JoinHandle<()> + Send>;

/// Engine
pub struct Engine {
  stores: BTreeMap<&'static str, Arc<DynStore>>,
  owners: Owners,
  start_queue: VecDeque<StartOperatorFn>,
  command_tx: Sender<CommandEvent>,
  command_rx: Receiver<CommandEvent>,
}

impl Engine {
  pub fn register_object<O>(&mut self)
  where
    O: ObjectDefinition,
  {
    // let store = Arc::new(Store::<O>::default());
    self
      .stores
      .entry(O::kind())
      .or_insert_with(|| Arc::new(Store::<O>::default()));
  }

  pub fn register_controller<O>(
    &mut self,
    controller: impl Controller<O>,
  ) -> Result<()>
  where
    O: ObjectDefinition,
  {
    let store = self.get_store::<O>()?;
    let command = self.command::<O>();

    let mut op = Operator::new(controller, command, store);
    let start_op = Box::new(|| tokio::spawn(async move { op.start().await }));

    self.start_queue.push_back(start_op);

    Ok(())
  }

  pub fn command<O>(&self) -> Command<O>
  where
    O: ObjectDefinition,
  {
    Command::<O>::new(self.command_tx.clone())
  }

  pub async fn start(mut self) {
    while let Some(start_fn) = self.start_queue.pop_front() {
      (start_fn)();
    }

    while let Ok(event) = self.command_rx.recv_async().await {
      println!("received command event: {:?}", event);

      if let Err(e) = self.handle_event(event).await {
        eprintln!("{e}");
      }
    }
  }

  async fn handle_event(&mut self, event: CommandEvent) -> Result<()> {
    match event {
      CommandEvent::InsertManifest(kind, manifest, owner) => {
        if let Some(owner) = owner {
          self.owners.own(owner, kind, manifest.name().to_owned())?;
        }
        self.get_store_kind(kind)?.insert(manifest)?;
      }
      CommandEvent::RemoveManifest(kind, name) => {
        if let Some(ownerships) = self.owners.remove_owner(&name) {
          for owned in ownerships {
            self.get_store_kind(owned.kind)?.remove(&owned.name)?;
          }
        }
        self.get_store_kind(kind)?.remove(&name)?;
      }
    }

    Ok(())
  }

  fn get_store<O>(&self) -> Result<Arc<Store<O>>>
  where
    O: ObjectDefinition,
  {
    self
      .stores
      .get(O::kind())
      .cloned()
      .ok_or_else(|| anyhow!("no store found for kind {}", O::kind()))
      .and_then(|s| s.as_store())
  }

  fn get_store_kind(&self, kind: &str) -> Result<Arc<DynStore>> {
    self
      .stores
      .get(kind)
      .cloned()
      .ok_or_else(|| anyhow!("no store found for kind {}", kind))
  }
}

impl Default for Engine {
  fn default() -> Self {
    let (command_tx, command_rx) = flume::unbounded();

    Self {
      stores: Default::default(),
      owners: Default::default(),
      start_queue: Default::default(),
      command_tx,
      command_rx,
    }
  }
}

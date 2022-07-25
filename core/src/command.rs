use std::{fmt::Debug, marker::PhantomData};

use anyhow::{bail, Result};
use flume::Sender;

use crate::{DynObjectManifest, ObjectDefinition, ObjectManifest, ObjectName};

/// CommandEvent
pub enum CommandEvent {
  InsertManifest(&'static str, Box<DynObjectManifest>, Option<ObjectName>),
  RemoveManifest(&'static str, ObjectName),
}

impl Debug for CommandEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let variant = match self {
      Self::InsertManifest(_, _, _) => "InsertManifest",
      Self::RemoveManifest(_, _) => "RemoveManifest",
    };

    write!(f, "{variant}")
  }
}

/// Command
pub struct Command<O>
where
  O: ObjectDefinition,
{
  sender: Sender<CommandEvent>,
  o: PhantomData<O>,
}

impl<O> Command<O>
where
  O: ObjectDefinition,
{
  pub fn new(sender: Sender<CommandEvent>) -> Self {
    Self {
      sender,
      o: PhantomData,
    }
  }

  pub fn insert_manifest(&self, manifest: ObjectManifest<O>) -> Result<()> {
    self.sender.send(CommandEvent::InsertManifest(
      O::kind(),
      Box::new(manifest),
      None,
    ))?;
    Ok(())
  }

  pub fn remove_manifest(&self, name: ObjectName) -> Result<()> {
    self
      .sender
      .send(CommandEvent::RemoveManifest(O::kind(), name))?;
    Ok(())
  }

  pub fn insert_owned_manifest<MO>(
    &self,
    owner: ObjectName,
    manifest: ObjectManifest<MO>,
  ) -> Result<()>
  where
    MO: ObjectDefinition,
  {
    if &owner == manifest.name() {
      bail!("an object can't own itself");
    }

    self.sender.send(CommandEvent::InsertManifest(
      MO::kind(),
      Box::new(manifest),
      Some(owner),
    ))?;
    Ok(())
  }
}

impl<O> Clone for Command<O>
where
  O: ObjectDefinition,
{
  fn clone(&self) -> Self {
    Self {
      sender: self.sender.clone(),
      o: self.o,
    }
  }
}

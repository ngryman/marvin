use std::{fmt::Debug, marker::PhantomData};

use anyhow::Result;
use flume::Sender;

use crate::{DynObjectManifest, ObjectDefinition, ObjectManifest, ObjectName};

/// CommandEvent
pub enum CommandEvent {
  InsertManifest(&'static str, Box<DynObjectManifest>),
  RemoveManifest(&'static str, ObjectName),
}

impl Debug for CommandEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let variant = match self {
      Self::InsertManifest(_, _) => "InsertManifest",
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
    self
      .sender
      .send(CommandEvent::InsertManifest(O::kind(), Box::new(manifest)))?;
    Ok(())
  }

  pub fn remove_manifest(&self, name: ObjectName) -> Result<()> {
    self
      .sender
      .send(CommandEvent::RemoveManifest(O::kind(), name))?;
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

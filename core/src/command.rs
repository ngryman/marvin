use anyhow::Result;
use flume::Sender;

use crate::{AnyObjectManifest, ObjectDefinition, ObjectManifest};

/// CommandEvent
pub enum CommandEvent {
  InsertManifest(Box<dyn AnyObjectManifest>),
}

/// Command
pub struct Command {
  sender: Sender<CommandEvent>,
}

impl Command {
  pub fn new(sender: Sender<CommandEvent>) -> Self {
    Self { sender }
  }

  pub fn insert_manifest<O>(&self, manifest: ObjectManifest<O>) -> Result<()>
  where
    O: ObjectDefinition,
  {
    self
      .sender
      .send(CommandEvent::InsertManifest(Box::new(manifest)))?;

    Ok(())
  }
}

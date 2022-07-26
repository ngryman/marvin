use std::{fmt::Debug, marker::PhantomData};

use anyhow::{bail, Result};
use flume::Sender;

use crate::{
  DynObjectManifest, ObjectDefinition, ObjectKind, ObjectManifest, ObjectName
};

/// CommandEvent
pub struct CommandEvent {
  pub action: CommandAction,
  pub ack: Option<catty::Sender<()>>,
}

/// CommandAction
pub enum CommandAction {
  InsertManifest(ObjectKind, Box<DynObjectManifest>, Option<ObjectName>),
  RemoveManifest(ObjectKind, ObjectName),
}

impl Debug for CommandAction {
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

  pub async fn insert_manifest(
    &self,
    manifest: ObjectManifest<O>,
  ) -> Result<()> {
    self
      .send_event(
        CommandAction::InsertManifest(O::kind(), Box::new(manifest), None),
        true,
      )
      .await
  }

  pub async fn insert_manifest_async(
    &self,
    manifest: ObjectManifest<O>,
  ) -> Result<()> {
    self
      .send_event(
        CommandAction::InsertManifest(O::kind(), Box::new(manifest), None),
        false,
      )
      .await
  }

  pub async fn remove_manifest(&self, name: ObjectName) -> Result<()> {
    self
      .send_event(CommandAction::RemoveManifest(O::kind(), name), true)
      .await
  }

  pub async fn remove_manifest_async(&self, name: ObjectName) -> Result<()> {
    self
      .send_event(CommandAction::RemoveManifest(O::kind(), name), false)
      .await
  }

  pub async fn insert_owned_manifest<MO>(
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

    self
      .send_event(
        CommandAction::InsertManifest(
          MO::kind(),
          Box::new(manifest),
          Some(owner),
        ),
        true,
      )
      .await
  }

  pub async fn insert_owned_manifest_async<MO>(
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

    self
      .send_event(
        CommandAction::InsertManifest(
          MO::kind(),
          Box::new(manifest),
          Some(owner),
        ),
        false,
      )
      .await
  }

  async fn send_event(&self, action: CommandAction, ack: bool) -> Result<()> {
    if ack {
      let (ack_tx, ack_rx) = catty::oneshot();
      self
        .sender
        .send_async(CommandEvent {
          action,
          ack: Some(ack_tx),
        })
        .await?;
      ack_rx.await?;
    } else {
      self
        .sender
        .send_async(CommandEvent { action, ack: None })
        .await?;
    }

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

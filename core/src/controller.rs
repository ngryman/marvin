use std::time::Duration;

use anyhow::{Error, Result};

use crate::{util::Safe, Command, ObjectDefinition, ObjectManifest};

/// Controller
#[allow(unused_variables)]
#[async_trait::async_trait]
pub trait Controller<O>: Safe
where
  O: ObjectDefinition,
{
  async fn admit_manifest(
    &self,
    manifest: ObjectManifest<O>,
  ) -> Result<ObjectManifest<O>> {
    Ok(manifest)
  }

  async fn initialize_state(
    &self,
    manifest: &ObjectManifest<O>,
  ) -> Result<O::State>;

  async fn terminate(&self, manifest: &ObjectManifest<O>) -> Result<()> {
    Ok(())
  }

  async fn should_reconcile(
    &self,
    next_manifest: &ObjectManifest<O>,
  ) -> Result<bool> {
    Ok(true)
  }

  async fn reconcile(
    &self,
    manifest: &ObjectManifest<O>,
    state: &mut O::State,
    command: &Command,
  ) -> Result<Option<Duration>> {
    Ok(None)
  }

  async fn reconcile_error(&self, e: Error) {
    eprintln!("{e}")
  }
}

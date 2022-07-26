use std::time::Duration;

use anyhow::Result;
use gusto_core::{
  Command, Controller, ObjectDefinition, ObjectManifest, ObjectMeta
};
use gusto_engine::Engine;

#[derive(Clone, Debug)]
#[allow(unused)]
struct FooProps {
  foo: bool,
}

type FooState = FooProps;

struct Foo;
impl ObjectDefinition for Foo {
  type Props = FooProps;
  type State = FooState;
}

#[derive(Default)]
struct FooController;

#[async_trait::async_trait]
impl Controller<Foo> for FooController {
  async fn initialize_state(
    &self,
    manifest: &ObjectManifest<Foo>,
  ) -> Result<FooState> {
    Ok(FooProps {
      foo: manifest.props.foo,
    })
  }

  async fn reconcile(
    &self,
    manifest: &ObjectManifest<Foo>,
    state: &mut FooProps,
    _command: &Command,
  ) -> Result<Option<Duration>> {
    state.foo = manifest.props.foo;
    dbg!(&state);
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(Some(Duration::ZERO))
  }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let mut engine = Engine::default();
  engine.register_object::<Foo>();
  engine.register_controller(FooController)?;

  let command = engine.command();

  let _ = tokio::try_join!(
    tokio::spawn(async move { engine.start().await }),
    tokio::spawn(async move {
      let mut manifest = ObjectManifest::<Foo> {
        meta: ObjectMeta {
          name: "proxy".into(),
        },
        props: FooProps { foo: true },
      };

      command.insert_manifest(manifest.clone()).await?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      manifest.props.foo = false;
      command.insert_manifest(manifest.clone()).await?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      manifest.props.foo = true;
      command.insert_manifest(manifest).await?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      command.remove_manifest::<Foo>("proxy".into()).await
    })
  )?;

  Ok(())
}

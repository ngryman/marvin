use std::time::Duration;

use anyhow::Result;
use gusto_core::{Controller, ObjectDefinition, ObjectManifest, ObjectMeta};
use gusto_engine::{Operator, Store};

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
  ) -> Result<Option<Duration>> {
    state.foo = manifest.props.foo;
    dbg!(&state);
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(Some(Duration::ZERO))
  }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let mut store = Store::default();
  let mut operator = Operator::new(FooController, store.clone());

  let _ = tokio::try_join!(
    tokio::spawn(async move { operator.start().await }),
    tokio::task::spawn(async move {
      let mut manifest = ObjectManifest::<Foo> {
        meta: ObjectMeta {
          name: "proxy".into(),
        },
        props: FooProps { foo: true },
      };

      store.insert(manifest.clone())?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      manifest.props.foo = false;
      store.insert(manifest.clone())?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      manifest.props.foo = true;
      store.insert(manifest)?;
      tokio::time::sleep(Duration::from_millis(500)).await;

      store.remove("proxy")
    })
  )?;

  Ok(())
}

use std::{collections::BTreeSet, time::Duration};

use anyhow::Result;
use gusto_core::{
  Command, Controller, ObjectDefinition, ObjectManifest, ObjectMeta, ObjectName
};
use gusto_engine::Engine;

#[derive(Clone, Debug)]
#[allow(unused)]
struct ChildProps {
  name: ObjectName,
}

struct Child;
impl ObjectDefinition for Child {
  type Props = ChildProps;
}

#[derive(Clone, Debug)]
#[allow(unused)]
struct ParentProps {
  children: Vec<ChildProps>,
}

#[derive(Default)]
struct ParentState {
  children: BTreeSet<ObjectName>,
}

struct Parent;
impl ObjectDefinition for Parent {
  type Props = ParentProps;
  type State = ParentState;
}

#[derive(Default)]
struct ParentController;

#[async_trait::async_trait]
impl Controller<Parent> for ParentController {
  async fn initialize_state(
    &self,
    _manifest: &ObjectManifest<Parent>,
  ) -> Result<ParentState> {
    Ok(ParentState::default())
  }

  async fn reconcile(
    &self,
    manifest: &ObjectManifest<Parent>,
    state: &mut ParentState,
    command: &Command<Parent>,
  ) -> Result<Option<Duration>> {
    for child_props in &manifest.props.children {
      if !state.children.contains(&child_props.name) {
        let child_manifest = ObjectManifest::<Child> {
          meta: ObjectMeta {
            name: child_props.name.clone(),
          },
          props: child_props.clone(),
        };

        command
          .insert_owned_manifest(manifest.name().to_owned(), child_manifest)?;
      }
    }

    Ok(Some(Duration::from_secs(1)))
  }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let mut engine = Engine::default();
  engine.register_object::<Parent>();
  engine.register_controller(ParentController)?;
  engine.register_object::<Child>();

  let command = engine.command::<Parent>();

  let _ = tokio::try_join!(
    tokio::spawn(async move { engine.start().await }),
    tokio::spawn(async move {
      let manifest = ObjectManifest::<Parent> {
        meta: ObjectMeta {
          name: "parent".into(),
        },
        props: ParentProps {
          children: vec![
            ChildProps {
              name: "Sara".into(),
            },
            ChildProps {
              name: "Michael".into(),
            },
          ],
        },
      };

      command.insert_manifest(manifest)?;
      tokio::time::sleep(Duration::from_secs(1)).await;

      command.remove_manifest("parent".into())
    })
  )?;

  Ok(())
}

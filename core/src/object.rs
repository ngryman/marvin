use std::any::Any;

use anyhow::{anyhow, Result};

use crate::util::Safe;

/// ObjectDefinition
pub trait ObjectDefinition: Safe {
  type Props: Props = ();
  type State: State = ();

  fn kind() -> &'static str {
    std::any::type_name::<Self>()
  }
}
impl ObjectDefinition for () {}

/// Props
pub trait Props: Clone + Safe {}
impl<T> Props for T where T: Clone + Safe {}

/// State
pub trait State: Safe {}
impl<T> State for T where T: Safe {}

/// ObjectName
pub type ObjectName = String;

/// ObjectMeta
#[derive(Clone)]
pub struct ObjectMeta {
  pub name: ObjectName,
}

/// ObjectManifest
pub struct ObjectManifest<O>
where
  O: ObjectDefinition,
{
  pub meta: ObjectMeta,
  pub props: O::Props,
}

impl<O> Clone for ObjectManifest<O>
where
  O: ObjectDefinition,
{
  fn clone(&self) -> Self {
    Self {
      meta: self.meta.clone(),
      props: self.props.clone(),
    }
  }
}

/// AnyObjectManifest
pub trait AnyObjectManifest: Any + Safe {}

impl<O> AnyObjectManifest for ObjectManifest<O> where O: ObjectDefinition {}

pub type DynObjectManifest = dyn AnyObjectManifest + Send + Sync + 'static;

impl DynObjectManifest {
  pub fn as_manifest<O>(self: Box<Self>) -> Result<Box<ObjectManifest<O>>>
  where
    O: ObjectDefinition,
  {
    (self as Box<dyn Any + Send + Sync>)
      .downcast()
      .map_err(|_| anyhow!("cannot downcast to {}", std::any::type_name::<O>()))
  }
}

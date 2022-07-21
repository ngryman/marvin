use std::any::Any;

use crate::util::Safe;

/// ObjectDefinition
pub trait ObjectDefinition: Safe {
  type Props: Props = ();
  type State: State = ();
}
impl ObjectDefinition for () {}

/// Props
pub trait Props: Clone + Safe {}
impl<T> Props for T where T: Clone + Safe {}

/// State
pub trait State: Safe {}
impl<T> State for T where T: Safe {}

/// ObjectMeta
#[derive(Clone)]
pub struct ObjectMeta {
  pub name: String,
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

impl dyn AnyObjectManifest {
  pub fn as_manifest<O>(&self) -> Option<&ObjectManifest<O>>
  where
    O: ObjectDefinition,
  {
    (self as &dyn Any).downcast_ref::<ObjectManifest<O>>()
  }
}

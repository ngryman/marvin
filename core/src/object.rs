use std::{any::Any, fmt::Display, ops::Deref};

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectName(String);

impl Display for ObjectName {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl Deref for ObjectName {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<String> for ObjectName {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl From<&str> for ObjectName {
  fn from(s: &str) -> Self {
    Self(s.to_owned())
  }
}

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

impl<O> ObjectManifest<O>
where
  O: ObjectDefinition,
{
  pub fn name(&self) -> &ObjectName {
    &self.meta.name
  }
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
pub trait AnyObjectManifest: Any + Safe {
  fn name(&self) -> &ObjectName;
}

impl<O> AnyObjectManifest for ObjectManifest<O>
where
  O: ObjectDefinition,
{
  fn name(&self) -> &ObjectName {
    ObjectManifest::name(self)
  }
}

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

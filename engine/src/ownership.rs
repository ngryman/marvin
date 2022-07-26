use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use gusto_core::{ObjectKind, ObjectName};

/// Owned
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Owned {
  pub kind: ObjectKind,
  pub name: ObjectName,
}

/// Owners
#[derive(Default)]
pub struct Owners {
  inner: BTreeMap<ObjectName, BTreeSet<Owned>>,
}

impl Owners {
  pub fn own(
    &mut self,
    owner: ObjectName,
    owned_kind: ObjectKind,
    owned_name: ObjectName,
  ) -> Result<()> {
    let owned = Owned {
      kind: owned_kind,
      name: owned_name,
    };

    match self.inner.entry(owner) {
      Entry::Vacant(entry) => {
        entry.insert(BTreeSet::from([owned]));
      }
      Entry::Occupied(mut entry) => {
        if !entry.get().contains(&owned) {
          entry.get_mut().insert(owned);
        } else {
          bail!("{} is already owned", owned.name);
        }
      }
    };

    Ok(())
  }

  pub fn remove_owner(
    &mut self,
    owner: &ObjectName,
  ) -> Option<BTreeSet<Owned>> {
    self.inner.remove(owner)
  }
}

use super::*;
use crate::{yaml::*, Error, Result};
use rustc_hash::FxHashMap;
use ryml::*;
use smartstring::alias::String;
use sprintf::sprintf;
use std::sync::RwLock;

static NAMES: &str = include_str!("../../data/botw_hashed_names.txt");
static NUMBERED_NAMES: &str = include_str!("../../data/botw_numbered_names.txt");

/// A table of names that is used to recover original names in binary parameter
/// archives which store only name hashes.
#[derive(Debug, Default)]
pub struct NameTable<'a> {
    names: RwLock<FxHashMap<u32, &'a str>>,
    owned_names: RwLock<FxHashMap<u32, String>>,
    numbered_names: Vec<&'a str>,
}

impl<'a> NameTable<'a> {
    pub fn new(botw_strings: bool) -> NameTable<'a> {
        if botw_strings {
            Self {
                names: RwLock::new(NAMES.lines().map(|n| (hash_name(n), n)).collect()),
                owned_names: RwLock::new(FxHashMap::default()),
                numbered_names: NUMBERED_NAMES.lines().collect(),
            }
        } else {
            Default::default()
        }
    }

    /// Add a known string to the name table.
    pub fn add_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        let hash = hash_name(&name);
        self.owned_names
            .write()
            .unwrap()
            .entry(hash)
            .or_insert(name);
    }

    /// Add a known string to the name table if you already know the hash (to
    /// avoid computing it).
    pub fn add_name_with_hash(&mut self, name: impl Into<String>, hash: u32) {
        self.owned_names
            .write()
            .unwrap()
            .entry(hash)
            .or_insert(name.into());
    }

    /// Add a known string to the name table.
    pub fn add_name_str<'s: 'a>(&'a mut self, name: &'s str) {
        let hash = hash_name(name);
        self.names.write().unwrap().entry(hash).or_insert(name);
    }

    /// Tries to guess the name that is associated with the given hash and index
    /// (of the parameter / object / list in its parent).
    ///
    /// The table is automatically updated with any newly found names if an
    /// indice-based guess was necessary.
    pub fn get_name(&'a self, hash: u32, index: usize, parent_hash: u32) -> Option<&'a str> {
        if let Some(name) = self.names.read().unwrap().get(&hash).copied().or_else(|| {
            self.owned_names
                .read()
                .unwrap()
                .get(&hash)
                .map(|s| s.as_str())
        }) {
            let (raw, len) = (name.as_ptr(), name.len());
            return Some(unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(raw, len))
            });
        }

        if let Some(parent_name) = self
            .names
            .read()
            .unwrap()
            .get(&parent_hash)
            .copied()
            .or_else(|| {
                self.owned_names
                    .read()
                    .unwrap()
                    .get(&parent_hash)
                    .map(|s| s.as_str())
            })
        // Try to guess the name from the parent structure if possible.
        {
            fn test_names<'a: 'b, 'b>(
                table: &'b NameTable<'a>,
                hash: u32,
                index: usize,
                prefix: &str,
            ) -> Option<&'a str> {
                let mut table = table.owned_names.write().unwrap();
                for i in index..(index + 1) {
                    for fmt in ["%s%d", "%s%02d", "%s%03d", "%s_%d", "%s_%02d", "%s_%03d"] {
                        if let Ok(name) = sprintf!(fmt, prefix, i)
                            && let candidate = hash_name(&name)
                            && candidate == hash
                        {
                            let name = table.entry(hash).or_insert(name.into());
                            let (raw, len) = (name.as_ptr(), name.len());
                            return Some(unsafe {
                                std::str::from_utf8_unchecked(std::slice::from_raw_parts(raw, len))
                            });
                        }
                    }
                }
                None
            }

            if let Some(found) = test_names(self, hash, index, parent_name)
                .or_else(|| test_names(self, hash, index, "Children"))
                .or_else(|| test_names(self, hash, index, "Child"))
                .or_else(|| {
                    // Sometimes the parent name is plural and the object names are singular.
                    for suffix in ["s", "es", "List"] {
                        if parent_name.ends_with(suffix) {
                            if let Some(found) = test_names(
                                self,
                                hash,
                                index,
                                &parent_name[..parent_name.len() - suffix.len()],
                            ) {
                                return Some(found);
                            }
                        }
                    }
                    None
                })
            {
                return Some(found);
            }
        }

        // Last resort: test all numbered names.
        for format in &self.numbered_names {
            for i in 0..(index + 2) {
                if let Ok(name) = sprintf!(&format, i)
                        && let candidate = hash_name(&name)
                        && candidate == hash
                    {
                        return Some(self.owned_names.write().unwrap().entry(hash).or_insert(name.into()));
                    }
            }
        }
        None
    }
}

struct Parser<'a>(Tree<'a>);

impl<'a> Parser<'a> {
    fn new(text: &str) -> Result<Self> {
        Ok(Self(Tree::parse(text)?))
    }

    // fn parse_parameter()

    fn parse(self) -> Result<ParameterIO> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_names() {
        let parent_hash: u32 = 2814088591;
        let hash: u32 = 2157271501;
        let index: usize = 35;
        // assert_eq!()
    }
}

use std::{
    borrow::Cow,
    collections::hash_map::{Entry, VacantEntry},
    fmt::Write,
    sync::Arc,
};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use super::*;

static NAMES: &str = include_str!("../../data/botw_hashed_names.txt");
static NUMBERED_NAMES: &str = include_str!("../../data/botw_numbered_names.txt");

/// Since there are basically no good runtime string formatting options in Rust,
/// we'll just do this instead.
struct ChildFormatIterator<'a, 'b> {
    string: &'a str,
    pos: usize,
    index: usize,
    buf: &'b mut std::string::String,
}

impl<'a, 'b> ChildFormatIterator<'a, 'b> {
    pub fn new(string: &'a str, pos: usize, buf: &'b mut std::string::String) -> Self {
        ChildFormatIterator {
            string,
            pos,
            index: 0,
            buf,
        }
    }
}

impl<'a, 'b> Iterator for ChildFormatIterator<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear(); // Clear the buffer for reuse

        use std::fmt::Write;
        let result = match self.index {
            0 => write!(self.buf, "{}{}", self.string, self.pos),
            1 => write!(self.buf, "{}{:02}", self.string, self.pos),
            2 => write!(self.buf, "{}{:03}", self.string, self.pos),
            3 => write!(self.buf, "{}_{}", self.string, self.pos),
            4 => write!(self.buf, "{}_{:02}", self.string, self.pos),
            5 => write!(self.buf, "{}_{:03}", self.string, self.pos),
            _ => return None,
        };

        self.index += 1;
        result.ok().map(|_| hash_name(self.buf.as_str()))
    }
}

impl ExactSizeIterator for ChildFormatIterator<'_, '_> {
    fn len(&self) -> usize {
        6
    }
}

#[inline(always)]
fn format_number(format: &str, pos: usize, buf: &mut std::string::String) {
    buf.clear();
    match format {
        "%d" | "%u" => write!(buf, "{}", pos),
        "%02d" | "%02u" => write!(buf, "{:02}", pos),
        "%03d" => write!(buf, "{:03}", pos),
        "%04d" => write!(buf, "{:04}", pos),
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
    .expect("Format failure")
}

fn format_numbered_name(name: &str, pos: usize, buf: &mut std::string::String) {
    for fmt in ["%d", "%02d", "%03d", "%04d", "%u", "%02u"] {
        if name.contains(fmt) {
            buf.clear();
            let mut split = name.split(fmt);
            format_number(fmt, pos, buf);
            buf.insert_str(0, unsafe { split.next().unwrap_unchecked() });
            if let Some(suf) = split.next() {
                buf.push_str(suf);
            }
            return;
        }
    }
    unsafe { core::hint::unreachable_unchecked() }
}

macro_rules! free_cow {
    ($cow:expr, $life:tt) => {{
        let cow = $cow as *const _;
        unsafe { &*(cow as *const Cow<$life, str>) }
    }};
}

/// A table of names that is used to recover original names in binary parameter
/// archives which store only name hashes.
///
/// Because binary parameter archives only store CRC32 hashes of structure
/// names, recovering the original names – which is useful for converting
/// archives to a human-readable format – requires the use of a name table.
///
/// When serializing to YAML, by default roead will use a table that contains
/// strings from Breath of the Wild’s executable.
#[derive(Debug, Default)]
pub struct NameTable<'a> {
    names: RwLock<FxHashMap<u32, Cow<'a, str>>>,
    numbered_names: Vec<&'a str>,
}

impl<'a> NameTable<'a> {
    /// Create a new name table, optionally including default BOTW strings.
    pub fn new(botw_strings: bool) -> NameTable<'a> {
        if botw_strings {
            Self {
                names: RwLock::new(NAMES.lines().map(|n| (hash_name(n), n.into())).collect()),
                numbered_names: NUMBERED_NAMES.lines().collect(),
            }
        } else {
            Default::default()
        }
    }

    /// Add a known string to the name table.
    pub fn add_name(&self, name: impl Into<Cow<'a, str>>) {
        let name = name.into();
        let hash = hash_name(&name);
        self.names.write().entry(hash).or_insert(name);
    }

    /// Add a known string to the name table if you already know the hash (to
    /// avoid computing it).
    pub fn add_name_with_hash(&self, name: impl Into<Cow<'a, str>>, hash: u32) {
        self.names
            .write()
            .entry(hash)
            .or_insert_with(|| name.into());
    }

    /// Add a known string to the name table.
    pub fn add_name_str<'s: 'a>(&'a self, name: &'s str) {
        let hash = hash_name(name);
        self.names
            .write()
            .entry(hash)
            .or_insert_with(|| name.into());
    }

    /// Tries to guess the name that is associated with the given hash and index
    /// (of the parameter / object / list in its parent).
    ///
    /// The table is automatically updated with any newly found names if an
    /// indice-based guess was necessary.
    pub fn get_name(&self, hash: u32, index: usize, parent_hash: u32) -> Option<&Cow<'_, str>> {
        fn test_names<'a: 'b, 'b, 'c>(
            entry: VacantEntry<'b, u32, Cow<'a, str>>,
            hash: u32,
            index: usize,
            prefix: &str,
            buf: &'c mut std::string::String,
        ) -> std::result::Result<&'b Cow<'a, str>, VacantEntry<'b, u32, Cow<'a, str>>> {
            for i in index..(index + 1) {
                for guess_hash in ChildFormatIterator::new(prefix, i, buf) {
                    if guess_hash == hash {
                        let name = entry.insert(buf.to_string().into());
                        return Ok(free_cow!(name, 'a));
                    }
                }
            }
            Err(entry)
        }

        let mut names = self.names.write();
        let parent_name = names.get(&parent_hash).map(|c| free_cow!(c, 'a));
        match names.entry(hash) {
            Entry::Occupied(entry) => Some(free_cow!(entry.get(), 'a)),
            Entry::Vacant(entry) => {
                let mut entry = entry;
                let mut guess_buffer = std::string::String::with_capacity(256);
                if let Some(parent_name) = parent_name
                // Try to guess the name from the parent structure if possible.
                {
                    let guess = test_names(entry, hash, index, parent_name, &mut guess_buffer)
                        .or_else(|entry| {
                            test_names(entry, hash, index, "Children", &mut guess_buffer)
                        })
                        .or_else(|entry| test_names(entry, hash, index, "Child", &mut guess_buffer))
                        .or_else(|mut entry| {
                            // Sometimes the parent name is plural and the object names are
                            // singular.
                            for suffix in ["s", "es", "List"] {
                                if let Some(singular) = parent_name.strip_suffix(suffix) {
                                    let guess =
                                        test_names(entry, hash, index, singular, &mut guess_buffer);
                                    match guess {
                                        Ok(found) => return Ok(found),
                                        Err(ret_entry) => entry = ret_entry,
                                    }
                                }
                            }
                            Err(entry)
                        });
                    match guess {
                        Ok(found) => return Some(free_cow!(found, 'a)),
                        Err(ret_entry) => {
                            entry = ret_entry;
                        }
                    }
                }
                // Last resort: test all numbered names.
                for format in &self.numbered_names {
                    for i in 0..(index + 2) {
                        format_numbered_name(format, i, &mut guess_buffer);
                        if hash_name(&guess_buffer) == hash {
                            let name = entry.insert(guess_buffer.to_string().into());
                            return Some(free_cow!(name, 'a));
                        }
                    }
                }
                None
            }
        }
    }
}

static DEFAULT_NAME_TABLE: Lazy<Arc<NameTable<'static>>> =
    Lazy::new(|| Arc::new(NameTable::new(true)));

/// Returns the default instance of the name table, which is automatically
/// populated with Breath of the Wild strings. It is initialised on first use
/// and has interior mutability.
pub fn get_default_name_table() -> &'static Lazy<Arc<NameTable<'static>>> {
    &DEFAULT_NAME_TABLE
}

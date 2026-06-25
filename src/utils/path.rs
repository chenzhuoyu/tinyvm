use std::path::{Path, PathBuf};

use crate::Maybe;

pub trait LibPathNormalizeExt {
    fn normalize(&self) -> Maybe<PathBuf>;
}

impl<P: AsRef<Path>> LibPathNormalizeExt for P {
    fn normalize(&self) -> Maybe<PathBuf> {
        Ok(soft_canonicalize::soft_canonicalize(self)?)
    }
}

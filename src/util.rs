use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::Error;

/// Copy static files recursively from `from` to `to`.
///
/// <https://stackoverflow.com/questions/26958489/>
pub(crate) fn copy_static(from: &Path, to: &Path) -> Result<(), Error> {
    let from_skip = from.components().count();
    let mut stack = vec![from.to_path_buf()];
    while let Some(curr) = stack.pop() {
        let src: PathBuf = curr.components().skip(from_skip).collect();
        let dest = &to.join(src);
        if dest.metadata().is_err() {
            fs::create_dir_all(dest)?;
        }
        for entry in curr.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(name) => {
                        println!("  -- {}", path.display());
                        fs::copy(&path, &dest.join(name))?;
                    }
                    None => unreachable!(),
                }
            }
        }
    }
    Ok(())
}

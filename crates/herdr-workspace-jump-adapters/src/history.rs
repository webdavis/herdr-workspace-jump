use std::fs;
use std::path::{Path, PathBuf};

use herdr_workspace_jump_application::WorkspaceHistory;
use herdr_workspace_jump_domain::Mru;

pub struct FileWorkspaceHistory {
    path: PathBuf,
}

impl FileWorkspaceHistory {
    pub fn at(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}

impl WorkspaceHistory for FileWorkspaceHistory {
    fn read(&self) -> Mru {
        read_at(&self.path)
    }
    fn write(&mut self, mru: &Mru) {
        write_at(&self.path, mru);
    }
}

fn read_at(path: &Path) -> Mru {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut lines = content.lines();
    Mru {
        current: lines.next().unwrap_or_default().trim().to_string(),
        previous: lines.next().unwrap_or_default().trim().to_string(),
    }
}

fn write_at(path: &Path, mru: &Mru) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, format!("{}\n{}\n", mru.current, mru.previous));
}

#[cfg(test)]
mod tests;

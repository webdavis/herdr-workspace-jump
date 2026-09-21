use std::fmt;

/// One workspace as the configuration declares it, before its ids are derived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDeclaration {
    pub label: String,
    pub directory: String,
    /// The key that selects this workspace in the pick popup, when declared.
    pub key: Option<String>,
}

/// One workspace a chord can jump to, with the action id herdr registers it
/// under and the key that selects it in the pick popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTarget {
    pub action_id: String,
    pub label: String,
    pub directory: String,
    pub pick_key: char,
}

/// Why a declared workspace list cannot become a set of actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetError {
    NoWorkspaces,
    EmptyLabel,
    EmptyDirectory {
        label: String,
    },
    CollidingActionId {
        first: String,
        second: String,
        action_id: String,
    },
    UnusablePickKey {
        label: String,
        key: String,
    },
    LabelHasNoDefaultPickKey {
        label: String,
    },
    CollidingPickKey {
        first: String,
        second: String,
        key: char,
    },
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWorkspaces => write!(formatter, "the [workspaces] table declares no workspace"),
            Self::EmptyLabel => write!(formatter, "a workspace label is empty"),
            Self::EmptyDirectory { label } => {
                write!(formatter, "the workspace {label} has an empty directory")
            }
            Self::CollidingActionId {
                first,
                second,
                action_id,
            } => write!(
                formatter,
                "the labels {first} and {second} both derive the action id {action_id}"
            ),
            Self::UnusablePickKey { label, key } => write!(
                formatter,
                "the workspace {label} declares the pick key {key:?}, which is not one printable ASCII character"
            ),
            Self::LabelHasNoDefaultPickKey { label } => write!(
                formatter,
                "the workspace {label} starts with a character that cannot be a pick key, so it has to declare one"
            ),
            Self::CollidingPickKey { first, second, key } => write!(
                formatter,
                "the workspaces {first} and {second} both pick on {key}"
            ),
        }
    }
}

/// Derive herdr's action id from a workspace label.
///
/// Lowercased, with every character outside `a-z0-9` replaced by an underscore,
/// because an action id is what a `plugin_action` keybinding names.
pub fn action_id(label: &str) -> String {
    let mut id = String::from("jump_");
    for character in label.chars() {
        let lowered = character.to_ascii_lowercase();
        id.push(match lowered {
            'a'..='z' | '0'..='9' => lowered,
            _ => '_',
        });
    }
    id
}

/// The key that selects a workspace in the pick popup.
///
/// A declared key wins; otherwise the label's first character, lowercased. The
/// popup prints the key and reads it back as one keystroke, so it has to be one
/// character the terminal can both show and deliver.
fn pick_key(declaration: &WorkspaceDeclaration) -> Result<char, TargetError> {
    let Some(declared) = &declaration.key else {
        return declaration
            .label
            .chars()
            .next()
            .map(|first| first.to_ascii_lowercase())
            .filter(char::is_ascii_graphic)
            .ok_or_else(|| TargetError::LabelHasNoDefaultPickKey {
                label: declaration.label.clone(),
            });
    };
    let mut characters = declared.chars();
    characters
        .next()
        .filter(|_| characters.next().is_none())
        .filter(char::is_ascii_graphic)
        .ok_or_else(|| TargetError::UnusablePickKey {
            label: declaration.label.clone(),
            key: declared.clone(),
        })
}

/// Turn declared workspaces into jump targets, in the order given.
pub fn jump_targets<Declarations>(
    declarations: Declarations,
) -> Result<Vec<JumpTarget>, TargetError>
where
    Declarations: IntoIterator<Item = WorkspaceDeclaration>,
{
    let mut targets: Vec<JumpTarget> = Vec::new();
    for declaration in declarations {
        if declaration.label.is_empty() {
            return Err(TargetError::EmptyLabel);
        }
        if declaration.directory.is_empty() {
            return Err(TargetError::EmptyDirectory {
                label: declaration.label,
            });
        }
        let action_id = action_id(&declaration.label);
        if let Some(existing) = targets.iter().find(|target| target.action_id == action_id) {
            return Err(TargetError::CollidingActionId {
                first: existing.label.clone(),
                second: declaration.label,
                action_id,
            });
        }
        let pick_key = pick_key(&declaration)?;
        if let Some(existing) = targets.iter().find(|target| target.pick_key == pick_key) {
            return Err(TargetError::CollidingPickKey {
                first: existing.label.clone(),
                second: declaration.label,
                key: pick_key,
            });
        }
        targets.push(JumpTarget {
            action_id,
            label: declaration.label,
            directory: declaration.directory,
            pick_key,
        });
    }
    if targets.is_empty() {
        return Err(TargetError::NoWorkspaces);
    }
    Ok(targets)
}

#[cfg(test)]
mod tests;

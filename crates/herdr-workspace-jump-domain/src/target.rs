use std::fmt;

/// One workspace a chord can jump to, with the action id herdr registers it under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTarget {
    pub action_id: String,
    pub label: String,
    pub directory: String,
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

/// Turn declared label and directory pairs into jump targets, in the order given.
pub fn jump_targets<Declarations>(
    declarations: Declarations,
) -> Result<Vec<JumpTarget>, TargetError>
where
    Declarations: IntoIterator<Item = (String, String)>,
{
    let mut targets: Vec<JumpTarget> = Vec::new();
    for (label, directory) in declarations {
        if label.is_empty() {
            return Err(TargetError::EmptyLabel);
        }
        if directory.is_empty() {
            return Err(TargetError::EmptyDirectory { label });
        }
        let action_id = action_id(&label);
        if let Some(existing) = targets.iter().find(|target| target.action_id == action_id) {
            return Err(TargetError::CollidingActionId {
                first: existing.label.clone(),
                second: label,
                action_id,
            });
        }
        targets.push(JumpTarget {
            action_id,
            label,
            directory,
        });
    }
    if targets.is_empty() {
        return Err(TargetError::NoWorkspaces);
    }
    Ok(targets)
}

#[cfg(test)]
mod tests;

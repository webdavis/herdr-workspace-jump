use super::*;

fn declaration(label: &str, directory: &str) -> (String, String) {
    (label.to_string(), directory.to_string())
}

#[test]
fn action_id_reproduces_the_ids_the_committed_manifest_registered() {
    assert_eq!(action_id("Ivy"), "jump_ivy");
    assert_eq!(action_id("casually-concerned"), "jump_casually_concerned");
    assert_eq!(action_id("justdavis-ansible"), "jump_justdavis_ansible");
    assert_eq!(action_id("homelab"), "jump_homelab");
}

#[test]
fn action_id_replaces_every_character_outside_a_to_z_and_zero_to_nine() {
    assert_eq!(action_id("a b/c"), "jump_a_b_c");
    assert_eq!(action_id("v2.1"), "jump_v2_1");
    // A multi-byte character is one replacement, not one per byte.
    assert_eq!(action_id("café"), "jump_caf_");
}

#[test]
fn jump_targets_keeps_the_declared_order_and_derives_every_id() {
    let targets = jump_targets(vec![
        declaration("Ivy", "~/workspaces/Ivy"),
        declaration("netpulse", "/opt/netpulse"),
    ])
    .expect("two well formed workspaces");

    assert_eq!(
        targets,
        vec![
            JumpTarget {
                action_id: "jump_ivy".to_string(),
                label: "Ivy".to_string(),
                directory: "~/workspaces/Ivy".to_string(),
            },
            JumpTarget {
                action_id: "jump_netpulse".to_string(),
                label: "netpulse".to_string(),
                directory: "/opt/netpulse".to_string(),
            },
        ]
    );
}

#[test]
fn jump_targets_refuses_an_empty_declaration_list() {
    assert_eq!(jump_targets(vec![]), Err(TargetError::NoWorkspaces));
}

#[test]
fn jump_targets_refuses_an_empty_label_or_an_empty_directory() {
    assert_eq!(
        jump_targets(vec![declaration("", "/opt/project")]),
        Err(TargetError::EmptyLabel)
    );
    assert_eq!(
        jump_targets(vec![declaration("netpulse", "")]),
        Err(TargetError::EmptyDirectory {
            label: "netpulse".to_string()
        })
    );
}

#[test]
fn jump_targets_refuses_two_labels_that_derive_the_same_action_id() {
    let collision = jump_targets(vec![
        declaration("my-project", "/opt/one"),
        declaration("my_project", "/opt/two"),
    ]);

    assert_eq!(
        collision,
        Err(TargetError::CollidingActionId {
            first: "my-project".to_string(),
            second: "my_project".to_string(),
            action_id: "jump_my_project".to_string(),
        })
    );
    assert!(
        collision
            .unwrap_err()
            .to_string()
            .contains("jump_my_project"),
        "the message names the colliding id"
    );
}

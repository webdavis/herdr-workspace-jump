use super::*;

fn declaration(label: &str, directory: &str) -> WorkspaceDeclaration {
    WorkspaceDeclaration {
        label: label.to_string(),
        directory: directory.to_string(),
        key: None,
    }
}

fn keyed(label: &str, directory: &str, key: &str) -> WorkspaceDeclaration {
    WorkspaceDeclaration {
        key: Some(key.to_string()),
        ..declaration(label, directory)
    }
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
                pick_key: 'i',
            },
            JumpTarget {
                action_id: "jump_netpulse".to_string(),
                label: "netpulse".to_string(),
                directory: "/opt/netpulse".to_string(),
                pick_key: 'n',
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

#[test]
fn a_declared_pick_key_wins_over_the_default() {
    let targets = jump_targets(vec![keyed("homelab", "/srv/homelab", "L")]).expect("one workspace");

    assert_eq!(targets[0].pick_key, 'L');
}

#[test]
fn the_default_pick_key_is_the_labels_first_character_lowercased() {
    let targets = jump_targets(vec![
        declaration("Ivy", "/opt/ivy"),
        declaration("2fa", "/opt/2fa"),
        declaration("-dash", "/opt/dash"),
    ])
    .expect("three workspaces");

    let keys: Vec<char> = targets.iter().map(|target| target.pick_key).collect();
    assert_eq!(keys, vec!['i', '2', '-']);
}

#[test]
fn a_label_whose_first_character_cannot_be_a_key_needs_its_own_key() {
    assert_eq!(
        jump_targets(vec![declaration("émile", "/opt/emile")]),
        Err(TargetError::LabelHasNoDefaultPickKey {
            label: "émile".to_string()
        })
    );
    let rescued =
        jump_targets(vec![keyed("émile", "/opt/emile", "e")]).expect("the declared key rescues it");
    assert_eq!(rescued[0].pick_key, 'e');
}

#[test]
fn a_pick_key_must_be_exactly_one_printable_ascii_character() {
    for rejected in ["", "ab", "é", " ", "\t"] {
        assert_eq!(
            jump_targets(vec![keyed("homelab", "/srv/homelab", rejected)]),
            Err(TargetError::UnusablePickKey {
                label: "homelab".to_string(),
                key: rejected.to_string(),
            }),
            "{rejected:?} is not a usable pick key"
        );
    }
}

#[test]
fn two_workspaces_resolving_to_the_same_pick_key_are_refused_by_name() {
    let collision = jump_targets(vec![
        declaration("dotfiles", "/opt/dotfiles"),
        declaration("damnit", "/opt/damnit"),
    ]);

    assert_eq!(
        collision,
        Err(TargetError::CollidingPickKey {
            first: "dotfiles".to_string(),
            second: "damnit".to_string(),
            key: 'd',
        })
    );
    let sentence = collision.unwrap_err().to_string();
    assert!(sentence.contains("dotfiles"), "{sentence}");
    assert!(sentence.contains("damnit"), "{sentence}");
    assert!(!sentence.contains('\n'), "one sentence: {sentence}");
}

#[test]
fn a_declared_key_resolves_a_default_collision() {
    let targets = jump_targets(vec![
        declaration("dotfiles", "/opt/dotfiles"),
        keyed("damnit", "/opt/damnit", "a"),
    ])
    .expect("the declared key breaks the tie");

    assert_eq!(targets[1].pick_key, 'a');
}

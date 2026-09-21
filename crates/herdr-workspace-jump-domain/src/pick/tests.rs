use super::*;
use crate::{WorkspaceDeclaration, jump_targets};

fn targets(declarations: Vec<(&str, Option<&str>)>) -> Vec<JumpTarget> {
    let declared = declarations
        .into_iter()
        .map(|(label, key)| WorkspaceDeclaration {
            label: label.to_string(),
            directory: format!("/opt/{label}"),
            key: key.map(str::to_string),
        })
        .collect::<Vec<_>>();
    jump_targets(declared).expect("well formed workspaces")
}

fn three_workspaces() -> Vec<JumpTarget> {
    targets(vec![
        ("Ivy", None),
        ("homelab", None),
        ("dotfiles", Some("f")),
    ])
}

#[test]
fn the_menu_lists_every_workspace_in_order_and_ends_with_the_cancel_line() {
    assert_eq!(
        render_menu(&three_workspaces()),
        "i  Ivy\nh  homelab\nf  dotfiles\nesc  cancel\n"
    );
}

#[test]
fn a_workspace_key_selects_that_workspace() {
    let workspaces = three_workspaces();

    assert_eq!(
        decide_pick(&workspaces, PickKey::Character('f')),
        Pick::Selected(&workspaces[2])
    );
    assert_eq!(
        decide_pick(&workspaces, PickKey::Character('i')),
        Pick::Selected(&workspaces[0])
    );
}

#[test]
fn the_key_that_selects_is_the_exact_declared_character() {
    let workspaces = three_workspaces();

    // The popup reads one keystroke, so an uppercase press is a different key.
    assert_eq!(
        decide_pick(&workspaces, PickKey::Character('I')),
        Pick::Unbound
    );
}

#[test]
fn escape_q_and_an_interrupt_all_cancel() {
    let workspaces = three_workspaces();

    for cancelling in [PickKey::Escape, PickKey::Interrupt, PickKey::Character('q')] {
        assert_eq!(decide_pick(&workspaces, cancelling), Pick::Cancelled);
    }
}

#[test]
fn an_unbound_character_and_an_unreadable_key_do_nothing() {
    let workspaces = three_workspaces();

    assert_eq!(
        decide_pick(&workspaces, PickKey::Character('z')),
        Pick::Unbound
    );
    assert_eq!(decide_pick(&workspaces, PickKey::Other), Pick::Unbound);
}

#[test]
fn a_workspace_picking_on_q_is_reached_rather_than_shadowed_by_cancel() {
    let workspaces = targets(vec![("queue", None)]);

    assert_eq!(
        decide_pick(&workspaces, PickKey::Character('q')),
        Pick::Selected(&workspaces[0])
    );
    assert_eq!(decide_pick(&workspaces, PickKey::Escape), Pick::Cancelled);
}

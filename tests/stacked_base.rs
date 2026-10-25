//! Stacked branches: `b` again in `branch` flips the base between the default branch and the parent.

mod common;

use common::Repo;
use herdr_reviewr::app::App;
use herdr_reviewr::handle_key;
use herdr_reviewr::keymap::Keymap;
use herdr_reviewr::model::Scope;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;

fn press(app: &mut App, code: KeyCode) {
    handle_key(app, KeyEvent::from(code), Rect::new(0, 0, 120, 40), &Keymap::default()).unwrap();
}

fn changed(app: &App) -> Vec<String> {
    app.entries.iter().map(|e| e.path.clone()).collect()
}

#[test]
fn b_again_flips_a_stacked_worktree_between_its_parent_and_the_default_branch() {
    let r = Repo::init();
    r.write("base.rs", "base\n");
    r.commit_all("init");
    r.git(&["checkout", "-q", "-b", "feature-a"]);
    r.write("a.rs", "a\n");
    r.commit_all("a");
    // A sibling on the same parent is as far behind HEAD, so it must not read as the parent.
    r.git(&["branch", "feature-c"]);
    let wt = r.add_worktree("feature-b");
    let at = wt.path().to_str().unwrap();
    std::fs::write(wt.path().join("b.rs"), "b\n").unwrap();
    r.git(&["-C", at, "add", "-A"]);
    r.git(&["-C", at, "commit", "-q", "-m", "b"]);
    r.git(&["checkout", "-q", "feature-c"]);
    r.write("c.rs", "c\n");
    r.commit_all("c");

    let mut app = App::new(wt.path().to_path_buf(), Scope::Uncommitted, None);
    app.reload().unwrap();

    press(&mut app, KeyCode::Char('b'));
    assert_eq!(app.scope, Scope::Branch);
    assert_eq!(changed(&app), ["a.rs", "b.rs"], "the default branch first");

    press(&mut app, KeyCode::Char('b'));
    assert_eq!(app.status, "base: feature-a");
    assert_eq!(changed(&app), ["b.rs"], "only this branch's own commits");

    // The flip is the worktree's base pick, so a fresh pane opens on it.
    let mut fresh = App::new(wt.path().to_path_buf(), Scope::Branch, None);
    fresh.reload().unwrap();
    assert_eq!(changed(&fresh), ["b.rs"]);

    press(&mut app, KeyCode::Char('b'));
    assert_eq!(app.status, "base: main");
    assert_eq!(changed(&app), ["a.rs", "b.rs"]);

    // The B picker tags and promotes the parent under the default.
    press(&mut app, KeyCode::Char('B'));
    let rows = &app.base_picker.as_ref().unwrap().rows;
    assert!(rows[0].parent() && rows[0].name() == "feature-a");
}

#[test]
fn b_again_on_a_branch_straight_off_the_default_has_no_parent_to_flip_to() {
    let r = Repo::init();
    r.write("base.rs", "base\n");
    r.commit_all("init");
    r.git(&["checkout", "-q", "-b", "feature"]);
    r.write("f.rs", "f\n");
    r.commit_all("f");
    let mut app = App::new(r.path_buf(), Scope::Branch, None);
    app.reload().unwrap();

    press(&mut app, KeyCode::Char('b'));
    assert_eq!(app.status, "no parent branch: comparing against main");
    assert_eq!(changed(&app), ["f.rs"]);
}

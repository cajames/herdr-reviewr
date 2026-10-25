//! Switching the reviewed worktree: the picker lists the clone's worktrees, and each keeps its comments.

mod common;

use std::path::Path;

use common::{Repo, UnifiedRow as _, app_on};
use herdr_reviewr::app::{App, Focus, Mode};
use herdr_reviewr::config::Config;
use herdr_reviewr::keymap::Keymap;
use herdr_reviewr::{handle_key, switch_worktree};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;

fn press(app: &mut App, code: KeyCode) {
    handle_key(app, KeyEvent::from(code), Rect::new(0, 0, 120, 40), &Keymap::default()).unwrap();
}

fn config(repo: &Path) -> Config {
    Config {
        repo: repo.to_path_buf(),
        base: None,
        theme: None,
        wrap: None,
        plugin_config_dir: None,
        removed_flag: None,
    }
}

fn comment(app: &mut App, text: &str) {
    app.focus = Focus::Diff;
    app.diff_cursor = app.visible.iter().position(|row| row.text().contains("two")).unwrap();
    app.start_comment();
    for ch in text.chars() {
        app.input_push(ch);
    }
    app.submit_comment();
}

#[test]
fn a_picked_worktree_reviews_its_own_changes_and_each_worktree_keeps_its_comments() {
    let r = Repo::init();
    r.write("a.rs", "one\n");
    r.commit_all("init");
    r.write("a.rs", "two\n");
    let wt = r.add_worktree("agent");
    std::fs::write(wt.path().join("b.rs"), "two\n").unwrap();
    let cfg = config(r.path());
    let mut parked = std::collections::HashMap::new();

    let mut app = app_on(&r);
    comment(&mut app, "main note");

    press(&mut app, KeyCode::Char('W'));
    assert_eq!(app.mode, Mode::WorktreePick);
    let names: Vec<(bool, Option<&str>)> =
        app.worktree_rows.iter().map(|row| (row.current, row.branch.as_deref())).collect();
    assert_eq!(names.len(), 2, "the main checkout and the linked worktree");
    assert!(names[0].0, "the reviewed worktree leads when no agent works elsewhere");
    assert_eq!(names[1].1, Some("agent"));

    // Picking the current one closes the picker and stays.
    press(&mut app, KeyCode::Enter);
    assert_eq!((app.mode.clone(), app.switch_to.clone()), (Mode::Normal, None));

    press(&mut app, KeyCode::Char('W'));
    press(&mut app, KeyCode::Char('2'));
    press(&mut app, KeyCode::Enter);
    let target = app.switch_to.take().expect("another worktree ends the loop");

    let mut app = switch_worktree(&cfg, &mut app, &target, &mut parked);
    let paths: Vec<&str> = app.entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, ["b.rs"], "the linked worktree's own changes");
    assert!(app.store.is_empty(), "the main checkout's comment stays with it");
    assert_eq!(app.unsent(), 1, "a parked comment still holds a quit");
    comment(&mut app, "wt note");

    // The picker shows the parked comment on its worktree; switching back restores it.
    press(&mut app, KeyCode::Char('W'));
    let main = app.worktree_rows.iter().find(|row| !row.current).unwrap();
    assert_eq!(main.parked, 1);
    let main = main.path.clone();
    app.close_worktree_picker();

    let app = switch_worktree(&cfg, &mut app, &main, &mut parked);
    let texts: Vec<&str> = app.store.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(texts, ["main note"]);
    assert_eq!(app.unsent(), 2);
}

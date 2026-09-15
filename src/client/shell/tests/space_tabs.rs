use super::*;

fn tab(
    tab_id: &str,
    workspace_id: &str,
    number: usize,
    label: &str,
    custom: bool,
) -> ClientShellTab {
    ClientShellTab {
        tab_id: tab_id.into(),
        workspace_id: workspace_id.into(),
        number,
        label: label.into(),
        custom_label: custom,
        zoomed: false,
        focused: false,
        agent_status: AgentStatus::Idle,
    }
}

/// `ws_1` (focused) with a focused auto tab and a custom-named tab, plus
/// `ws_2` holding a single auto-named tab.
fn snapshot_with_tabs() -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    let mut second = snapshot.workspaces[0].clone();
    second.workspace_id = "ws_2".into();
    second.active_tab_id = "tab_3".into();
    second.number = 2;
    second.label = "other-space".into();
    second.focused = false;
    snapshot.workspaces.push(second);
    snapshot.tabs = vec![
        tab("tab_1", "ws_1", 1, "1", false),
        tab("tab_2", "ws_1", 2, "build", true),
        tab("tab_3", "ws_2", 1, "1", false),
    ];
    snapshot.tabs[0].focused = true;
    snapshot
}

fn state_with_tabs(show_tabs: bool) -> ClientShellState {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.spaces.show_tabs = show_tabs;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(snapshot_with_tabs()));
    state.set_pane_surface(surface());
    state
}

fn frame_text(frame: &FrameData) -> String {
    frame_rows(frame).join("\n")
}

#[test]
fn hidden_tab_rows_leave_the_spaces_sidebar_untouched() {
    let mut disabled = state_with_tabs(false);
    let plain = disabled.compose(106, 24).expect("sidebar without tab rows");
    let plain_rows = frame_rows(&plain);
    assert!(disabled.hits.workspace_tabs.is_empty());
    assert!(!frame_text(&plain).contains("├─"));
    assert!(!frame_text(&plain).contains("└─"));

    let mut enabled = state_with_tabs(true);
    let shown = enabled.compose(106, 24).expect("sidebar with tab rows");
    assert!(!enabled.hits.workspace_tabs.is_empty());
    assert_ne!(plain_rows, frame_rows(&shown));
}

#[test]
fn workspace_tab_rows_render_tree_prefixes_glyphs_and_focus() {
    let mut state = state_with_tabs(true);
    let frame = state.compose(106, 24).expect("sidebar with tab rows");
    let rows = frame_rows(&frame);

    let hits = state
        .hits
        .workspace_tabs
        .iter()
        .map(|hit| (hit.workspace_id.as_str(), hit.tab_id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        hits,
        [("ws_1", "tab_1"), ("ws_1", "tab_2")],
        "a lone auto-named tab gets no row"
    );

    let first = state.hits.workspace_tabs[0].rect;
    let second = state.hits.workspace_tabs[1].rect;
    assert_eq!(second.y, first.y + 1);
    let first_row = &rows[usize::from(first.y)];
    let second_row = &rows[usize::from(second.y)];
    assert!(first_row.contains("├─ "), "{first_row:?}");
    assert!(second_row.contains("└─ "), "{second_row:?}");
    assert!(second_row.contains("build"), "{second_row:?}");

    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    let (glyph_x, _) = cell_symbol_position(&frame, first, "○");
    assert_eq!(glyph_x, first.x + 6, "tab rows nest one level deeper");
    assert_eq!(
        buffer[(glyph_x, first.y)].fg,
        state.config.palette.green,
        "tab status glyph keeps its status color"
    );
    let label_cell = &buffer[(glyph_x + 2, first.y)];
    assert_eq!(label_cell.fg, state.config.palette.text);
    assert!(label_cell.modifier.contains(Modifier::BOLD));
    assert_eq!(label_cell.bg, state.config.palette.active_row_bg);
    let other_label = &buffer[(glyph_x + 2, second.y)];
    assert_eq!(other_label.fg, state.config.palette.subtext0);
    assert!(!other_label.modifier.contains(Modifier::BOLD));
}

#[test]
fn clicking_a_tab_row_focuses_the_tab_without_moving_workspaces() {
    let mut state = state_with_tabs(true);
    let mut snapshot = snapshot_with_tabs();
    snapshot.tabs.push(tab("tab_4", "ws_2", 2, "review", true));
    state.set_snapshot(Box::new(snapshot));
    state.compose(106, 24).expect("sidebar with tab rows");

    let hit = state
        .hits
        .workspace_tabs
        .iter()
        .find(|hit| hit.tab_id == "tab_4")
        .expect("tab row for the unfocused workspace")
        .rect;
    let mut outcome = ClientShellInput::default();
    state.handle_mouse(
        crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: hit.x + 7,
            row: hit.y,
            modifiers: KeyModifiers::NONE,
        },
        &mut outcome,
    );
    assert!(state.workspace_press.is_none(), "no workspace drag starts");
    assert!(matches!(
        outcome.actions.as_slice(),
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                crate::api::schema::Method::TabFocus(target) if target.tab_id == "tab_4"
            )
    ));
}

#[test]
fn tab_chevron_folds_tab_rows_and_persists_the_choice() {
    let path = std::env::temp_dir().join(format!(
        "herdr-client-shell-space-tabs-{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.spaces.show_tabs = true;
    let mut state = ClientShellState::new(config.with_preferences_path(path.clone()));
    state.set_snapshot(Box::new(snapshot_with_tabs()));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 24).expect("expanded tab rows");

    let toggle = state
        .hits
        .workspaces
        .iter()
        .find(|hit| hit.workspace_id == "ws_1")
        .and_then(|hit| hit.tab_toggle.clone())
        .expect("tab chevron")
        .0;
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    assert_eq!(buffer[(toggle.x, toggle.y)].symbol(), "▾");
    assert!(state
        .hits
        .workspaces
        .iter()
        .all(|hit| hit.workspace_id != "ws_2" || hit.tab_toggle.is_none()));

    let mut outcome = ClientShellInput::default();
    state.handle_mouse(
        crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: toggle.x,
            row: toggle.y,
            modifiers: KeyModifiers::NONE,
        },
        &mut outcome,
    );
    assert!(outcome.repaint);
    let collapsed = state.compose(106, 24).expect("collapsed tab rows");
    assert!(state.hits.workspace_tabs.is_empty());
    assert!(!frame_rows(&collapsed)
        .iter()
        .any(|row| row.contains("└─") || row.contains("├─")));

    let buffer = collapsed
        .to_ratatui_buffer()
        .expect("frame should reconstruct");
    assert_eq!(buffer[(toggle.x, toggle.y)].symbol(), "▸");

    let stored = std::fs::read_to_string(&path).expect("stored preferences");
    assert!(stored.contains("collapsed_tab_workspaces"));
    let reloaded_config =
        ClientShellConfig::from_config(&Config::default()).with_preferences_path(path.clone());
    let reloaded = ClientShellState::new(reloaded_config);
    assert_eq!(
        reloaded.collapsed_tab_workspaces,
        HashSet::from(["ws_1".to_owned()])
    );
    std::fs::remove_file(path).expect("remove preferences");
}

#[test]
fn legacy_preferences_without_tab_collapses_still_load() {
    let preferences: preferences::ClientChromePreferences =
        serde_json::from_str(r#"{"collapsed_groups":["/repo"],"sidebar_width":24}"#)
            .expect("legacy client chrome preferences");
    assert_eq!(preferences.sidebar_width, Some(24));
    assert!(preferences.collapsed_tab_workspaces.is_empty());
    assert!(preferences.remote_collapsed_tab_workspaces.is_empty());
}

#[test]
fn workspace_context_menu_toggles_tab_rows_only_when_enabled() {
    let mut off = state_with_tabs(false);
    off.compose(106, 24).expect("sidebar without tab rows");
    off.open_workspace_context_menu("ws_1".into(), 0, 0);
    assert!(match off.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => menu
            .items()
            .iter()
            .all(|item| item.action != ClientContextMenuAction::ToggleTabs),
        _ => panic!("workspace context menu"),
    });

    let mut state = state_with_tabs(true);
    state.compose(106, 24).expect("sidebar with tab rows");
    state.open_workspace_context_menu("ws_1".into(), 0, 0);
    let index = match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => {
            let index = menu
                .items()
                .iter()
                .position(|item| item.action == ClientContextMenuAction::ToggleTabs)
                .expect("tab toggle item");
            assert_eq!(menu.items()[index].label, "Hide tabs");
            index
        }
        _ => panic!("workspace context menu"),
    };
    state.activate_context_menu_item(index, &mut ClientShellInput::default());
    assert!(state.workspace_tabs_are_collapsed(&ClientEndpointId::Local, "ws_1"));

    state.open_workspace_context_menu("ws_1".into(), 0, 0);
    match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => {
            assert!(menu.items().iter().any(|item| item.label == "Show tabs"
                && item.action == ClientContextMenuAction::ToggleTabs))
        }
        _ => panic!("workspace context menu"),
    }

    // A workspace with only one auto-named tab never offers the item.
    state.open_workspace_context_menu("ws_2".into(), 0, 0);
    assert!(match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => menu
            .items()
            .iter()
            .all(|item| item.action != ClientContextMenuAction::ToggleTabs),
        _ => panic!("workspace context menu"),
    });
}

#[test]
fn right_click_on_a_tab_row_opens_the_tab_context_menu() {
    let mut state = state_with_tabs(true);
    state.compose(106, 24).expect("sidebar with tab rows");
    let hit = state
        .hits
        .workspace_tabs
        .iter()
        .find(|hit| hit.tab_id == "tab_2")
        .expect("tab row")
        .rect;
    let mut outcome = ClientShellInput::default();
    state.handle_mouse(
        crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: hit.x + 7,
            row: hit.y,
            modifiers: KeyModifiers::NONE,
        },
        &mut outcome,
    );
    match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => {
            assert!(matches!(
                &menu.target,
                ClientContextMenuTarget::Tab { tab_id, .. } if tab_id == "tab_2"
            ));
            assert!(menu.items().iter().any(
                |item| item.label == "Rename" && item.action == ClientContextMenuAction::Rename
            ));
        }
        _ => panic!("tab context menu"),
    }
}

#[test]
fn worktree_child_tab_rows_nest_below_their_own_workspace() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.spaces.show_tabs = true;
    let mut state = ClientShellState::new(config);
    let mut snapshot = snapshot_with_tabs();
    snapshot.workspaces[0].worktree = Some(ClientShellWorktree {
        key: "repo".into(),
        label: "repo".into(),
        is_linked_worktree: false,
    });
    snapshot.workspaces[1].branch = Some("worktree/feature".into());
    snapshot.workspaces[1].worktree = Some(ClientShellWorktree {
        key: "repo".into(),
        label: "repo".into(),
        is_linked_worktree: true,
    });
    snapshot.tabs = vec![
        tab("tab_1", "ws_1", 1, "1", false),
        tab("tab_3", "ws_2", 1, "1", false),
        tab("tab_4", "ws_2", 2, "review", true),
    ];
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    let frame = state
        .compose(106, 24)
        .expect("worktree group with tab rows");
    let rows = frame_rows(&frame);

    let parent = state
        .hits
        .workspaces
        .iter()
        .find(|hit| hit.workspace_id == "ws_1")
        .expect("parent workspace row");
    assert!(
        parent.group_toggle.is_some(),
        "parent keeps its worktree group chevron"
    );
    assert!(
        parent.tab_toggle.is_none(),
        "the worktree chevron is not replaced"
    );

    let child_tabs = state
        .hits
        .workspace_tabs
        .iter()
        .filter(|hit| hit.workspace_id == "ws_2")
        .map(|hit| hit.rect)
        .collect::<Vec<_>>();
    assert_eq!(child_tabs.len(), 2);
    assert!(state
        .hits
        .workspace_tabs
        .iter()
        .all(|hit| hit.workspace_id == "ws_2"));
    let child = state
        .hits
        .workspaces
        .iter()
        .find(|hit| hit.workspace_id == "ws_2")
        .expect("child workspace row");
    assert!(child.indented);
    assert_eq!(child_tabs[0].y, child.rect.bottom());
    let (glyph_x, _) = cell_symbol_position(&frame, child_tabs[0], "○");
    assert_eq!(
        glyph_x,
        child_tabs[0].x + 9,
        "worktree children nest one level deeper still"
    );
    assert!(rows[usize::from(child_tabs[1].y)].contains("review"));
}

#[test]
fn remote_tab_rows_activate_their_machine() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.spaces.show_tabs = true;
    let mut state = ClientShellState::new(config);
    let profile =
        SavedSshEndpoint::new("Build", "dev@build.example", "agents").expect("saved SSH profile");
    let remote_id = ClientEndpointId::Ssh(profile.id.clone());
    state.set_endpoint_catalog(&[profile]);
    state.set_endpoint_status(&remote_id, ClientEndpointStatus::Online);
    state.set_snapshot(Box::new(snapshot_with_tabs()));
    state.set_pane_surface(surface());
    let mut remote = snapshot_with_tabs();
    remote.boot_id = "remote-boot".into();
    remote.workspaces[0].label = "remote-workspace".into();
    remote.tabs[1].label = "remote-build".into();
    state.set_endpoint_snapshot(&remote_id, Box::new(remote));
    let frame = state.compose(106, 44).expect("multi machine sidebar");
    assert!(frame_text(&frame).contains("remote-build"));

    let hit = state
        .hits
        .workspace_tabs
        .iter()
        .find(|hit| hit.endpoint_id == remote_id && hit.tab_id == "tab_2")
        .expect("remote tab row")
        .rect;
    let mut outcome = ClientShellInput::default();
    state.handle_mouse(
        crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: hit.x + 9,
            row: hit.y,
            modifiers: KeyModifiers::NONE,
        },
        &mut outcome,
    );
    assert!(matches!(
        outcome.actions.as_slice(),
        [ClientShellAction::ActivateEndpoint { endpoint_id, target: Some(ClientEndpointFocusTarget::Tab(tab_id)) }]
            if endpoint_id == &remote_id && tab_id == "tab_2"
    ));
}

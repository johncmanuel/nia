use super::{App, GroupLayout, Tab, TabGroup};
use crate::config;
use crate::document::Document;
use iced::widget::{
    markdown,
    pane_grid::{self, Axis, Configuration, Node},
};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

pub(super) struct Session {
    paths: Vec<Option<PathBuf>>,
    reading: Vec<bool>,
    active_group: usize,
    groups: Vec<SavedGroup>,
}

struct SavedGroup {
    tabs: Vec<u64>,
    active: u64,
    layout: Configuration<u64>,
}

pub(crate) fn save_session(app: &App) -> io::Result<()> {
    let directory = config::directory()?;
    save_to(app, &directory.join("layout.conf"))
}

pub(super) fn load() -> io::Result<Option<Session>> {
    load_from(&config::directory()?.join("layout.conf"))
}

pub(super) fn save_to(app: &App, path: &Path) -> io::Result<()> {
    let mut text = format!("nia-layout-3\n{}\n{}\n", app.tabs.len(), app.active_group);
    for tab in &app.tabs {
        match &tab.document.path {
            Some(path) => {
                for byte in path.to_string_lossy().as_bytes() {
                    write!(text, "{byte:02x}").map_err(io::Error::other)?;
                }
                text.push('\n');
            }
            None => text.push_str("-\n"),
        }
    }
    for tab in &app.tabs {
        text.push_str(if tab.reading.is_some() { "R\n" } else { "W\n" });
    }
    writeln!(text, "{}", app.groups.len()).map_err(io::Error::other)?;
    for (index, group) in app.groups.iter().enumerate() {
        let (panes, active_pane) = if index == app.active_group {
            (&app.panes, app.active_pane)
        } else {
            let saved = group.inactive.as_ref().ok_or_else(invalid_layout)?;
            (&saved.panes, saved.active_pane)
        };
        let active = panes.get(active_pane).ok_or_else(invalid_layout)?;
        let active = tab_index(app, *active)?;
        writeln!(text, "{}\n{active}", group.tabs.len()).map_err(io::Error::other)?;
        for id in &group.tabs {
            writeln!(text, "{}", tab_index(app, *id)?).map_err(io::Error::other)?;
        }
        write_node(app, panes, panes.layout(), &mut text)?;
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid layout path"))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, text)?;
    fs::rename(temporary, path)
}

fn tab_index(app: &App, id: u64) -> io::Result<usize> {
    app.tabs
        .iter()
        .position(|tab| tab.id == id)
        .ok_or_else(invalid_layout)
}

fn write_node(
    app: &App,
    panes: &pane_grid::State<u64>,
    node: &Node,
    text: &mut String,
) -> io::Result<()> {
    match node {
        Node::Pane(pane) => {
            let id = panes.get(*pane).ok_or_else(invalid_layout)?;
            writeln!(text, "P:{}", tab_index(app, *id)?).map_err(io::Error::other)?;
        }
        Node::Split {
            axis, ratio, a, b, ..
        } => {
            let axis = if *axis == Axis::Vertical { 'V' } else { 'H' };
            writeln!(text, "S:{axis}:{ratio}").map_err(io::Error::other)?;
            write_node(app, panes, a, text)?;
            write_node(app, panes, b, text)?;
        }
    }
    Ok(())
}

pub(super) fn load_from(path: &Path) -> io::Result<Option<Session>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    decode(&text).map(Some).ok_or_else(invalid_layout)
}

pub(super) fn decode(text: &str) -> Option<Session> {
    let mut lines = text.lines();
    let version = lines.next()?;
    if !matches!(version, "nia-layout-1" | "nia-layout-2" | "nia-layout-3") {
        return None;
    }
    let count: usize = lines.next()?.parse().ok()?;
    if !(1..=100).contains(&count) {
        return None;
    }
    let selection: usize = lines.next()?.parse().ok()?;
    let mut paths = Vec::with_capacity(count);
    for _ in 0..count {
        let line = lines.next()?;
        if line == "-" {
            paths.push(None);
            continue;
        }
        let (bytes, remainder) = line.as_bytes().as_chunks::<2>();
        if !remainder.is_empty() {
            return None;
        }
        let decoded: Vec<u8> = bytes
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
            .collect::<Option<_>>()?;
        paths.push(Some(PathBuf::from(String::from_utf8(decoded).ok()?)));
    }
    let reading = if version == "nia-layout-3" {
        (0..count)
            .map(|_| match lines.next()? {
                "R" => Some(true),
                "W" => Some(false),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?
    } else {
        vec![false; count]
    };
    let (active_group, groups) = if version == "nia-layout-1" {
        if selection >= count {
            return None;
        }
        let allowed = vec![true; count];
        let mut seen = vec![false; count];
        let mut remaining = count * 2 - 1;
        let layout = read_node(&mut lines, &allowed, &mut seen, &mut remaining)?;
        if !seen[selection] {
            return None;
        }
        (
            0,
            vec![SavedGroup {
                tabs: (0..count as u64).collect(),
                active: selection as u64,
                layout,
            }],
        )
    } else {
        let group_count: usize = lines.next()?.parse().ok()?;
        if group_count == 0 || group_count > count || selection >= group_count {
            return None;
        }
        let mut owned = vec![false; count];
        let mut groups = Vec::with_capacity(group_count);
        for _ in 0..group_count {
            let tab_count: usize = lines.next()?.parse().ok()?;
            if tab_count == 0 || tab_count > count {
                return None;
            }
            let active: usize = lines.next()?.parse().ok()?;
            let mut allowed = vec![false; count];
            let mut tabs = Vec::with_capacity(tab_count);
            for _ in 0..tab_count {
                let index: usize = lines.next()?.parse().ok()?;
                if index >= count || owned[index] {
                    return None;
                }
                owned[index] = true;
                allowed[index] = true;
                tabs.push(index as u64);
            }
            let mut seen = vec![false; count];
            let mut remaining = tab_count * 2 - 1;
            let layout = read_node(&mut lines, &allowed, &mut seen, &mut remaining)?;
            if active >= count || !seen[active] {
                return None;
            }
            groups.push(SavedGroup {
                tabs,
                active: active as u64,
                layout,
            });
        }
        if owned.iter().any(|owned| !owned) {
            return None;
        }
        (selection, groups)
    };
    if lines.next().is_some() {
        return None;
    }
    Some(Session {
        paths,
        reading,
        active_group,
        groups,
    })
}

fn read_node<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    allowed: &[bool],
    seen: &mut [bool],
    remaining: &mut usize,
) -> Option<Configuration<u64>> {
    *remaining = remaining.checked_sub(1)?;
    let line = lines.next()?;
    if let Some(index) = line.strip_prefix("P:") {
        let index: usize = index.parse().ok()?;
        if index >= allowed.len() || !allowed[index] || seen[index] {
            return None;
        }
        seen[index] = true;
        return Some(Configuration::Pane(index as u64));
    }
    let (axis, ratio) = line.strip_prefix("S:")?.split_once(':')?;
    let axis = match axis {
        "V" => Axis::Vertical,
        "H" => Axis::Horizontal,
        _ => return None,
    };
    let ratio: f32 = ratio.parse().ok()?;
    if !ratio.is_finite() || !(0.0..=1.0).contains(&ratio) {
        return None;
    }
    let a = Box::new(read_node(lines, allowed, seen, remaining)?);
    let b = Box::new(read_node(lines, allowed, seen, remaining)?);
    Some(Configuration::Split { axis, ratio, a, b })
}

fn invalid_layout() -> io::Error {
    io::Error::new(ErrorKind::InvalidData, "invalid saved layout")
}

pub(super) fn restore(app: &mut App, session: Session) {
    let mut missing = false;
    app.tabs = session
        .paths
        .into_iter()
        .zip(session.reading)
        .enumerate()
        .map(|(index, (path, reading))| {
            let document = match path {
                Some(path) => match Document::open(path) {
                    Ok(document) => document,
                    Err(error) => {
                        missing = true;
                        log::warn!("saved document could not reopen: {:?}", error.kind());
                        Document::new()
                    }
                },
                None => Document::new(),
            };
            let mut tab = Tab::new(index as u64, document);
            if reading {
                tab.reading = Some(markdown::Content::parse(&tab.document.contents));
            }
            tab
        })
        .collect();
    app.next_id = app.tabs.len() as u64;
    app.active_group = session.active_group;
    app.groups = session
        .groups
        .into_iter()
        .map(|saved| {
            let panes = pane_grid::State::with_configuration(saved.layout);
            let active_pane = panes
                .iter()
                .find_map(|(&pane, &id)| (id == saved.active).then_some(pane))
                .expect("validated group has its active pane");
            TabGroup {
                tabs: saved.tabs,
                inactive: Some(GroupLayout { panes, active_pane }),
            }
        })
        .collect();
    let active = app.groups[app.active_group]
        .inactive
        .take()
        .expect("validated active group has a layout");
    app.panes = active.panes;
    app.active_pane = active.active_pane;
    app.activate_visible_tab();
    if missing {
        app.notice = Some("Some previously open files could not be reopened.".into());
        log::warn!("saved layout has missing files");
    }
}

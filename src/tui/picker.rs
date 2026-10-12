use crate::adb::Adb;
use crate::adb::device::AdbDevice;
use crate::context::Context;
use crate::tui::notice::Notice;
use crate::tui::{STYLE_BG_TEXT, STYLE_ERROR, STYLE_HIGHLIGHT, STYLE_SELECTION};
use futures::StreamExt;
use indexmap::IndexMap;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Row, Table, TableState};
use std::ops::ControlFlow;
use std::time::Duration;
use tokio::select;
use tokio::time::MissedTickBehavior;

////////////////////////////////////////////////////////////////////////////////
const REFRESH_EVERY: Duration = Duration::from_secs(1);
const READY_STATE: &str = "device";
const COLUMNS: [(&str, Constraint); 7] = [
    ("", Constraint::Length(3)),
    ("ID", Constraint::Length(4)),
    ("Name", Constraint::Length(16)),
    ("Serial", Constraint::Length(24)),
    ("Product", Constraint::Length(16)),
    ("Model", Constraint::Length(20)),
    ("State", Constraint::Length(16)),
];
const HOTKEYS: [(&str, &str); 5] = [
    ("↑↓", "prev/next"),
    ("←→", "first/last"),
    ("enter", "toggle"),
    ("c", "copy claude launch command"),
    ("q", "quit"),
];

////////////////////////////////////////////////////////////////////////////////
pub struct DevicePicker {
    exposed: Context,
    mcp_url: String,
    entries: IndexMap<String, DeviceEntry>,
    table_state: TableState,
    adb_error: Notice,
    header_note: Notice,
}

struct DeviceEntry {
    device: AdbDevice,
    selected: bool,
    marked_for_removal: bool,
}

impl DevicePicker {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn new(exposed: Context, mcp_url: String) -> Self {
        Self {
            exposed,
            mcp_url,
            entries: Default::default(),
            table_state: Default::default(),
            adb_error: Default::default(),
            header_note: Default::default(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub async fn run(mut self, adb: &Adb) -> anyhow::Result<()> {
        let mut terminal = ratatui::init();
        let result = self.event_loop(&mut terminal, adb).await;
        ratatui::restore();
        result
    }

    ////////////////////////////////////////////////////////////////////////////////
    async fn event_loop(&mut self, terminal: &mut DefaultTerminal, adb: &Adb) -> anyhow::Result<()> {
        let mut events = EventStream::new();
        let mut refresh = tokio::time::interval(REFRESH_EVERY);
        refresh.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            self.update_entries(adb).await;

            terminal.draw(|frame| self.draw(frame))?;

            select! {
                Some(Ok(event)) = events.next() => {
                    if self.handle_event(event)?.is_break() {
                        return Ok(());
                    }
                }
                _ = refresh.tick() => {},
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn handle_event(&mut self, event: Event) -> anyhow::Result<ControlFlow<()>> {
        let Event::Key(key) = event else {
            return Ok(ControlFlow::Continue(()));
        };
        if key.kind != KeyEventKind::Press {
            return Ok(ControlFlow::Continue(()));
        }

        match key.code {
            KeyCode::Up => self.table_state.select_previous(),
            KeyCode::Down => self.table_state.select_next(),
            KeyCode::Left => self.table_state.select_first(),
            KeyCode::Right => self.table_state.select_last(),
            KeyCode::Enter => self.toggle(),
            KeyCode::Char('c') => self.copy_claude_launch_command(),
            KeyCode::Esc | KeyCode::Char('q') => return Ok(ControlFlow::Break(())),
            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }

    ////////////////////////////////////////////////////////////////////////////////
    async fn update_entries(&mut self, adb: &Adb) {
        for entry in self.entries.values_mut() {
            entry.marked_for_removal = true;
        }

        let devices = match adb.devices().await {
            Ok(infos) => infos,
            Err(e) => {
                self.adb_error.show(e.to_string());
                return;
            }
        };

        for device in devices {
            let device = self
                .entries
                .entry(device.serial.clone())
                .or_insert_with(|| DeviceEntry {
                    device: AdbDevice::new(adb.clone(), device),
                    selected: false,
                    marked_for_removal: false,
                });

            device.marked_for_removal = false;
        }

        self.entries.retain(|_, entry| !entry.marked_for_removal);
        self.publish();
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn toggle(&mut self) {
        let Some(index) = self.table_state.selected() else {
            return;
        };
        let Some((_, entry)) = self.entries.get_index_mut(index) else {
            return;
        };

        entry.selected = !entry.selected;
        self.publish();
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn publish(&self) {
        let devices = self
            .entries
            .values()
            .filter(|e| e.selected)
            .map(|e| e.device.clone())
            .collect();
        self.exposed.set(devices);
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn copy_claude_launch_command(&mut self) {
        let json = serde_json::json!({
            "mcpServers": {
                "mdev_adb": {
                    "type":"http",
                    "url": self.mcp_url,
                }
            }
        });

        let command = format!("claude --mcp-config '{json}' --allowedTools mcp__mdev_adb --strict-mcp-config");
        let result = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(command));

        let note = match result {
            Ok(()) => "Launch command copied to clipboard".to_owned(),
            Err(e) => format!("Failed to copy launch command: {e}"),
        };
        self.header_note.show(note);
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn draw(&mut self, frame: &mut Frame) {
        let constraints = [Constraint::Fill(1), Constraint::Length(3)];
        let [devices_area, hotkeys_area] = Layout::vertical(constraints).areas(frame.area());

        self.draw_devices(frame, devices_area);
        self.draw_hotkeys(frame, hotkeys_area);
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn draw_devices(&mut self, frame: &mut Frame, area: Rect) {
        let rows = self.entries.values().map(|e| {
            let optional = |value: &Option<String>| match value {
                Some(value) => Line::raw(value.clone()),
                None => Line::styled("-", STYLE_BG_TEXT),
            };
            let state_style = if e.device.state == READY_STATE {
                Style::new()
            } else {
                STYLE_ERROR
            };

            Row::new([
                Line::default().spans([
                    Span::raw("["),
                    Span::styled(if e.selected { "+" } else { " " }, STYLE_HIGHLIGHT),
                    Span::raw("]"),
                ]),
                Line::raw(e.device.id().to_string()),
                Line::raw(e.device.name.clone()),
                Line::raw(e.device.serial.clone()),
                optional(&e.device.product),
                optional(&e.device.model),
                Line::styled(e.device.state.clone(), state_style),
            ])
        });

        let headers = COLUMNS.map(|(header, _)| header);
        let widths = COLUMNS.map(|(_, width)| width);

        let title = Line::default().spans([
            Span::raw(" Select devices available via MCP at "),
            Span::styled(&self.mcp_url, STYLE_HIGHLIGHT),
            Span::raw(" "),
        ]);

        let mut block = Block::bordered().title(title);
        if let Some(note) = self.header_note.text() {
            block = block.title_bottom(Span::styled(format!(" {note} "), STYLE_HIGHLIGHT));
        } else if let Some(error) = self.adb_error.text() {
            block = block.title_bottom(Span::styled(format!(" {error} "), STYLE_ERROR));
        }

        let table = Table::new(rows, widths)
            .header(Row::new(headers).style(STYLE_HIGHLIGHT))
            .column_spacing(2)
            .block(block)
            .row_highlight_style(STYLE_SELECTION);

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn draw_hotkeys(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();
        let mut separator = "";

        for (key, action) in HOTKEYS {
            spans.push(Span::styled(separator, STYLE_BG_TEXT));
            spans.push(Span::styled(format!(" {key} "), STYLE_HIGHLIGHT));
            spans.push(Span::styled(format!("{action} "), STYLE_BG_TEXT));
            separator = " | ";
        }

        let block = Block::bordered().title(" Hotkeys ");
        let hotkeys = Paragraph::new(Line::from(spans)).block(block);

        frame.render_widget(hotkeys, area);
    }
}

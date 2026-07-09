use super::data::SessionRow;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub(super) struct PtyPane {
    session_id: String,
    pty_id: String,
    title: String,
    parser: vt100::Parser,
    stream: crate::pty::AttachStream,
    rows: u16,
    cols: u16,
    connected: bool,
}

impl PtyPane {
    pub(super) fn open(row: &SessionRow, pty_id: String) -> Result<Self> {
        let rows = 24;
        let cols = 80;
        let stream = crate::pty::attach_stream(&pty_id, rows, cols)?;
        Ok(Self {
            session_id: row.session_id.clone(),
            pty_id,
            title: pane_title(row),
            parser: vt100::Parser::new(rows, cols, 512),
            stream,
            rows,
            cols,
            connected: true,
        })
    }

    pub(super) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(super) fn pty_id(&self) -> &str {
        &self.pty_id
    }

    pub(super) fn title(&self) -> &str {
        &self.title
    }

    pub(super) fn connected(&self) -> bool {
        self.connected
    }

    pub(super) fn refresh_title(&mut self, row: &SessionRow) {
        self.title = pane_title(row);
    }

    pub(super) fn poll_output(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }
        let mut bytes = Vec::new();
        self.connected = self.stream.read_available(&mut bytes)?;
        if !bytes.is_empty() {
            self.parser.process(&bytes);
        }
        Ok(())
    }

    pub(super) fn write_input(&mut self, bytes: &[u8]) -> Result<bool> {
        if !self.connected {
            return Ok(false);
        }
        self.connected = self.stream.write_input(bytes)?;
        Ok(self.connected)
    }

    pub(super) fn resize(&mut self, rows: u16, cols: u16) {
        if rows == 0 || cols == 0 || (rows == self.rows && cols == self.cols) {
            return;
        }
        self.rows = rows;
        self.cols = cols;
        self.parser.screen_mut().set_size(rows, cols);
        let _ = self.stream.resize(rows, cols);
    }

    pub(super) fn lines(&self, width: u16, height: u16) -> Vec<Line<'static>> {
        screen_lines(self.parser.screen(), width, height)
    }

    pub(super) fn shutdown(&mut self) {
        self.stream.shutdown();
    }
}

fn pane_title(row: &SessionRow) -> String {
    format!("{} - {}", row.agent, row.title_with_activity())
}

fn screen_lines(screen: &vt100::Screen, width: u16, height: u16) -> Vec<Line<'static>> {
    (0..height)
        .map(|row| screen_line(screen, row, width))
        .collect()
}

fn screen_line(screen: &vt100::Screen, row: u16, width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let mut pending = String::new();
    let mut pending_style = Style::default();
    for col in 0..width {
        let Some(cell) = screen.cell(row, col) else {
            continue;
        };
        if cell.is_wide_continuation() {
            continue;
        }
        let style = cell_style(cell);
        let text = if cell.has_contents() {
            cell.contents()
        } else {
            " "
        };
        if style != pending_style && !pending.is_empty() {
            spans.push(Span::styled(std::mem::take(&mut pending), pending_style));
        }
        pending_style = style;
        pending.push_str(text);
    }
    if !pending.is_empty() {
        spans.push(Span::styled(pending, pending_style));
    }
    Line::from(spans)
}

fn cell_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default();
    if let Some(color) = vt_color(cell.fgcolor()) {
        style = style.fg(color);
    }
    if let Some(color) = vt_color(cell.bgcolor()) {
        style = style.bg(color);
    }
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.dim() {
        style = style.add_modifier(Modifier::DIM);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }
    style
}

fn vt_color(color: vt100::Color) -> Option<Color> {
    match color {
        vt100::Color::Default => None,
        vt100::Color::Idx(idx) => Some(Color::Indexed(idx)),
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
    }
}

pub(super) fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    if alt {
        out.push(0x1b);
    }
    let bytes: &[u8] = match key.code {
        KeyCode::Backspace => b"\x7f",
        KeyCode::Enter => b"\r",
        KeyCode::Left => b"\x1b[D",
        KeyCode::Right => b"\x1b[C",
        KeyCode::Up => b"\x1b[A",
        KeyCode::Down => b"\x1b[B",
        KeyCode::Home => b"\x1b[H",
        KeyCode::End => b"\x1b[F",
        KeyCode::PageUp => b"\x1b[5~",
        KeyCode::PageDown => b"\x1b[6~",
        KeyCode::Tab => b"\t",
        KeyCode::BackTab => b"\x1b[Z",
        KeyCode::Delete => b"\x1b[3~",
        KeyCode::Insert => b"\x1b[2~",
        KeyCode::Esc => b"\x1b",
        KeyCode::F(n) => return function_key(n, alt),
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            out.push(ctrl_byte(c)?);
            return Some(out);
        }
        KeyCode::Char(c) => {
            let mut buf = [0_u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            return Some(out);
        }
        _ => return None,
    };
    out.extend_from_slice(bytes);
    Some(out)
}

fn ctrl_byte(c: char) -> Option<u8> {
    match c {
        'a'..='z' => Some(c as u8 - b'a' + 1),
        'A'..='Z' => Some(c as u8 - b'A' + 1),
        '[' => Some(0x1b),
        '\\' => Some(0x1c),
        ']' => Some(0x1d),
        '^' => Some(0x1e),
        '_' => Some(0x1f),
        '?' => Some(0x7f),
        ' ' => Some(0),
        _ => None,
    }
}

fn function_key(n: u8, alt: bool) -> Option<Vec<u8>> {
    let seq = match n {
        1 => "\x1bOP",
        2 => "\x1bOQ",
        3 => "\x1bOR",
        4 => "\x1bOS",
        5 => "\x1b[15~",
        6 => "\x1b[17~",
        7 => "\x1b[18~",
        8 => "\x1b[19~",
        9 => "\x1b[20~",
        10 => "\x1b[21~",
        11 => "\x1b[23~",
        12 => "\x1b[24~",
        _ => return None,
    };
    let mut out = Vec::new();
    if alt {
        out.push(0x1b);
    }
    out.extend_from_slice(seq.as_bytes());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn encodes_basic_terminal_keys() {
        assert_eq!(
            encode_key(key(KeyCode::Char('x'), KeyModifiers::NONE)),
            Some(b"x".to_vec())
        );
        assert_eq!(
            encode_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(vec![3])
        );
        assert_eq!(
            encode_key(key(KeyCode::Enter, KeyModifiers::NONE)),
            Some(b"\r".to_vec())
        );
        assert_eq!(
            encode_key(key(KeyCode::Up, KeyModifiers::NONE)),
            Some(b"\x1b[A".to_vec())
        );
    }

    #[test]
    fn screen_lines_preserve_cell_color() {
        let mut parser = vt100::Parser::new(1, 4, 0);
        parser.process(b"\x1b[31mR\x1b[0m!");

        let lines = screen_lines(parser.screen(), 4, 1);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].content.as_ref(), "R");
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Indexed(1)));
        assert_eq!(lines[0].spans[1].content.as_ref(), "!  ");
        assert_eq!(lines[0].spans[1].style.fg, None);
    }
}

use crate::custom_terminal;
use crossterm::event::KeyEvent;
use ratatui::backend::Backend;
use ratatui::backend::ClearType;
use ratatui::backend::WindowSize;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use ratatui::text::Line;
use std::io;
use std::io::Write;
use std::time::Duration;
use tokio_stream::Empty;

pub(crate) const TARGET_FRAME_INTERVAL: Duration = Duration::from_millis(16);

pub(crate) fn running_in_vscode_terminal() -> bool {
    false
}

#[derive(Clone, Debug)]
pub enum TuiEvent {
    Key(KeyEvent),
    Paste(String),
    Resize,
    Draw,
}

pub(crate) type Terminal = custom_terminal::Terminal<BrowserBackend>;

pub(crate) struct Tui {
    frame_requester: FrameRequester,
    pub(crate) terminal: Terminal,
}

impl Default for Tui {
    fn default() -> Self {
        let backend = BrowserBackend::new(80, 24);
        let terminal = custom_terminal::Terminal::with_options(backend)
            .expect("browser terminal backend should initialize");
        Self {
            frame_requester: FrameRequester,
            terminal,
        }
    }
}

impl Tui {
    pub(crate) fn frame_requester(&self) -> FrameRequester {
        self.frame_requester.clone()
    }

    pub(crate) fn event_stream(&self) -> Empty<TuiEvent> {
        tokio_stream::empty()
    }

    pub(crate) fn draw(
        &mut self,
        height: u16,
        draw_fn: impl FnOnce(&mut custom_terminal::Frame),
    ) -> io::Result<()> {
        let mut area = self.terminal.viewport_area;
        if area.width == 0 || area.height == 0 {
            area = Rect::new(0, 0, 80, height.min(24));
            self.terminal.set_viewport_area(area);
        }
        self.terminal.draw(draw_fn)?;
        Ok(())
    }

    pub(crate) fn notify(&mut self, _message: String) {}

    pub(crate) fn insert_history_lines(&mut self, _lines: Vec<Line<'static>>) {
        self.frame_requester.schedule_frame();
    }
}

pub(crate) struct BrowserBackend {
    output: Vec<u8>,
    size: Size,
    cursor: Position,
}

impl BrowserBackend {
    fn new(width: u16, height: u16) -> Self {
        Self {
            output: Vec::new(),
            size: Size { width, height },
            cursor: Position { x: 0, y: 0 },
        }
    }
}

impl Write for BrowserBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Backend for BrowserBackend {
    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.cursor)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.cursor = position.into();
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn clear_region(&mut self, _clear_type: ClearType) -> io::Result<()> {
        Ok(())
    }

    fn append_lines(&mut self, _line_count: u16) -> io::Result<()> {
        Ok(())
    }

    fn scroll_region_up(
        &mut self,
        _region: std::ops::Range<u16>,
        _scroll_by: u16,
    ) -> io::Result<()> {
        Ok(())
    }

    fn scroll_region_down(
        &mut self,
        _region: std::ops::Range<u16>,
        _scroll_by: u16,
    ) -> io::Result<()> {
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size,
            pixels: self.size,
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FrameRequester;

impl FrameRequester {
    pub(crate) fn schedule_frame(&self) {}
    pub(crate) fn schedule_frame_in(&self, _delay: std::time::Duration) {}
}

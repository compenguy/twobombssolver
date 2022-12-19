use std::io::Write;

use crossterm::{cursor, event, style, terminal};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use crossterm::QueueableCommand;

use crate::coord::Coord;
use crate::errors::{TBError, Result};
use crate::terminal::settings::RenderSettings;
use crate::terminal::GameAction;

const MAX_MSGLOG_LINES: usize = 4;

pub struct CxTerm<W: Write> {
    writer: W,
    settings: RenderSettings,
    active_cell: Option<Coord>,
}

impl<W: Write> Drop for CxTerm<W> {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        self.writer.queue(cursor::Show);
        self.writer.queue(event::DisableMouseCapture);
        self.writer.queue(terminal::LeaveAlternateScreen);
        self.writer.flush();
    }
}

impl<W: Write> CxTerm<W> {
    pub fn new(settings: RenderSettings, writer: W) -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self {
            writer,
            settings,
            active_cell: None,
        })
    }

    pub fn reset_display(&mut self) -> Result<()> {
        self.writer
            .queue(terminal::EnterAlternateScreen)?
            .queue(event::EnableMouseCapture)?
            .queue(cursor::Hide)?
            .queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn draw_line(&mut self, coord: &Coord, text: &str) -> Result<()> {
        self.clear_line(coord)?;
        self.writer
            .queue(cursor::MoveTo(coord.x as u16, coord.y as u16))?
            .queue(style::Print(text.to_string()))?;
        Ok(())
    }

    pub fn clear_line(&mut self, coord: &Coord) -> Result<()> {
        self.writer
            .queue(cursor::MoveTo(coord.x as u16, coord.y as u16))?
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn write_title(&mut self, title: &str) -> Result<()> {
        self.draw_line(&self.settings.get_title_origin(), title)
    }

    pub fn write_status(&mut self, status: &str) -> Result<()> {
        self.draw_line(&self.settings.get_status_origin(), status)
    }

    pub fn clear_msglog(&mut self) -> Result<()> {
        // Clear *all* msglog lines
        for num in 0..MAX_MSGLOG_LINES {
            let coord = self.settings.get_msglog_origin() + Coord { x: 0, y: num };
            self.clear_line(&coord)?;
        }
        Ok(())
    }

    pub fn write_msglog(&mut self, status: &str) -> Result<()> {
        self.clear_msglog()?;
        // Write *up to* max number of msglog lines
        for (num, line) in status.lines().enumerate() {
            if num >= MAX_MSGLOG_LINES {
                break;
            }
            let coord = self.settings.get_msglog_origin() + Coord { x: 0, y: num };
            self.draw_line(&coord, line)?;
        }
        Ok(())
    }

    pub fn write_rendered_board_row(&mut self, row: usize, line_draw: &str) -> Result<()> {
        let mut board_row_origin = self.settings.get_board_origin();
        board_row_origin.y += row;
        self.draw_line(&board_row_origin, line_draw)
    }

    pub fn blink_cursor(&mut self, coord: &Coord) -> Result<()> {
        self.writer
            .queue(cursor::MoveTo(coord.x as u16, coord.y as u16))?
            .queue(cursor::Show)?
            .queue(cursor::EnableBlinking)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> Result<()> {
        self.writer.queue(cursor::Hide)?;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    pub fn get_active_board_cell(&self) -> Coord {
        self.active_cell.unwrap_or_default()
    }

    pub fn update_active_board_cell(&mut self, update: fn(&Coord) -> Coord) {
        let active = self.get_active_board_cell();
        let board_size = self.settings.get_board_size();
        let mut next = update(&active);

        if next.x > board_size - 1 {
            next.x = board_size - 1;
        }

        if next.y > board_size - 1 {
            next.y = board_size - 1;
        }

        self.active_cell = Some(next);
    }

    pub fn get_input_event(&mut self) -> Result<Event> {
        event::read().map_err(|_| TBError::InvalidGameInput)
    }

    pub fn get_game_action(&mut self) -> Result<GameAction> {
        let mut action: Option<GameAction> = None;
        while action.is_none() {
            let active_cell = self.get_active_board_cell();
            let term_coord = self.settings.cell_coord_to_term_coord(&active_cell);
            self.blink_cursor(&term_coord)?;
            self.commit()?;

            match self.get_input_event() {
                Ok(Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. })) => {
                    self.update_active_board_cell(|coord| *coord - Coord { x: 1, y: 0 });
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. })) => {
                    self.update_active_board_cell(|coord| *coord + Coord { x: 1, y: 0 });
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Up, kind: KeyEventKind::Press, .. })) => {
                    self.update_active_board_cell(|coord| *coord - Coord { x: 0, y: 1 });
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, .. })) => {
                    self.update_active_board_cell(|coord| *coord + Coord { x: 0, y: 1 });
                }
                Ok(Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::MarkTarget(active_cell));
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Char(' '), kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::MarkTarget(active_cell));
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Char('s'), kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::Solve);
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Char('S'), kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::Solve);
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Char('q'), kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::Quit);
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Char('Q'), kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::Quit);
                },
                Ok(Event::Key(KeyEvent { code: KeyCode::Esc, kind: KeyEventKind::Press, .. })) => {
                    action = Some(GameAction::Quit);
                },
                Ok(Event::Mouse(MouseEvent { kind: MouseEventKind::Down(_), column: term_x, row: term_y, .. })) => {
                    let game_coord = self.settings.term_coord_to_cell_coord(&Coord {
                        x: term_x as usize,
                        y: term_y as usize,
                    });
                    self.hide_cursor()?;
                    action = Some(GameAction::MarkTarget(game_coord));
                }
                _ => (),
            }
        }
        self.hide_cursor()?;
        self.commit()?;
        action.ok_or(TBError::InvalidGameInput)
    }
}

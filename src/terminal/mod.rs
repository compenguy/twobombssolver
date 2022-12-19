use log::debug;
use std::io::Write;

use crate::coord::Coord;
use crate::errors::Result;
use crate::twobombs::TBGame;

mod settings;
use settings::RenderSettings;

mod cxterm;
use cxterm::CxTerm;

pub enum GameAction {
    MarkTarget(Coord),
    Solve,
    Quit,
}

const INSTRUCTIONS: &str = r#"Press 'S', or 's' to solve.
Press 'Q', 'q', or <Esc> to quit.
To make a move:
1. Mouse click in square, or
2. Arrows to move and <Space> or <Enter> to select."#;

pub fn render_board<W: Write>(term: &mut CxTerm<W>, game: &mut TBGame, title: &str) -> Result<()> {
    let board_size = game.board.get_size();
    // Redraw board state
    term.write_title(title)?;

    for board_row in 0..board_size {
        let rendered_board_row: usize = board_row * 2;
        debug!(
            "Rendering board row: {} ({})",
            board_row, rendered_board_row
        );
        if board_row == 0 {
            term.write_rendered_board_row(rendered_board_row, &game.board.render_board_top())?;
        } else {
            term.write_rendered_board_row(rendered_board_row, &game.board.render_board_row_sep())?;
        }
        term.write_rendered_board_row(
            rendered_board_row + 1,
            &game.board.render_board_row(board_row),
        )?;
    }
    term.write_rendered_board_row(board_size * 2, &game.board.render_board_bottom())?;

    term.commit()?;
    Ok(())
}

pub fn play_game() -> Result<()> {
    let board_size: usize = 23;
    let mut game = TBGame::new_anysize(board_size);

    let mut term = CxTerm::new(RenderSettings::new(2, 4, board_size), std::io::stdout())?;
    debug!("Resetting display");
    term.reset_display()?;

    debug!("Starting game...");

    // We want this to be written once, and not refreshed with each loop
    term.write_msglog(INSTRUCTIONS)?;

    let mut title = String::from("Welcome to TwoBombs!");
    loop {
        render_board(&mut term, &mut game, title.as_str())?;
        match term.get_game_action()? {
            GameAction::MarkTarget(coord) => {
                game.board.mark_target(&coord);
                title = String::from(&format!("{} fields marked", game.board.cells.len()));
                term.commit()?;
            }
            GameAction::Solve => {
                if let Some(_) = game.board.find_bombs() {
                    title = String::from("Solution found!");
                    term.write_msglog("Solution found!")?;
                } else {
                    title = String::from("No solution found!");
                    term.write_msglog("No solution found!")?;
                }
                term.commit()?;
            }
            GameAction::Quit => return Ok(()),
        }
    }
}

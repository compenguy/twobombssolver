use std::num;

use thiserror::Error;

use crate::coord::Coord;

#[derive(Error, Debug)]
pub enum TBError {
    #[error("Invalid game input")]
    InvalidGameInput,
    #[error("Invalid cell position: {0}")]
    InvalidCellPosition(Coord),
    #[error("Logger error: {0}")]
    Logger(#[from] flexi_logger::FlexiLoggerError),
    #[error("Terminal error: {0}")]
    Terminal(#[from] crossterm::ErrorKind),
    #[error("Parse error: {0}")]
    BadParse(#[from] num::ParseIntError),
}

pub type Result<T> = std::result::Result<T, TBError>;

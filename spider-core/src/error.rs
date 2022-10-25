use thiserror::Error;

use std::result;

#[derive(Error, Debug)]
pub enum IError {
    #[error("internet error: {0}")]
    Internet(String),
    #[error("client error: {0}")]
    Client(String),
    #[error("open file error: {0}")]
    File(String),
    #[error("config file error: {0}")]
    Config(String),
    #[error("database error: {0}")]
    Database(String),
}

pub type IResult<T> = result::Result<T, IError>;

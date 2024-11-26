use error_code::ToErrorInfo;
use error_code_derive::ToErrorInfo;
use thiserror::Error;

#[derive(Debug, Error, ToErrorInfo)]
#[error_info(app_type = "http::StatusCode", prefix = "01")]
pub enum MyError {
    #[error("Invalid command: {0}")]
    #[error_info(code = "IC", app_code = "200")]
    InvalidCommand(String),

    #[error("Invalid argument: {0}")]
    #[error_info(code = "IA", app_code = "400", client_msg = "friendly message")]
    InvalidArgument(String),

    #[error("{0}")]
    #[error_info(code = "RE", app_code = "500")]
    RespError(#[from] std::io::Error),
}

fn main() -> anyhow::Result<()> {
    let error = MyError::InvalidCommand("hello".to_string());
    let info = error.to_error_info();

    println!("{:?}", info);
    Ok(())
}

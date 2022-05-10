use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Z-Machine internal error.")]
    ZMachineError(String)
}
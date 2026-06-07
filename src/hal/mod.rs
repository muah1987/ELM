//! Hardware Abstraction Layer (HAL) for ELM.
//! This layer isolates the cognitive engine from specific hardware registers.

pub mod pu;
pub mod rpi4;
pub mod discovery;

#[derive(Debug)]
pub enum HalError {
    BusError,
    Timeout,
    InvalidAddress,
    StorageFailure,
}

//! PU (Processing Unit) Abstraction Layer
//! Defines the core traits and registry for indexing native hardware components.

extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use core::ops::Deref;

/// Unique identifier for a Processing Unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PuId(pub u64);

/// Metadata describing a Processing Unit.
#[derive(Debug, Clone)]
pub struct PuInfo {
    pub name: &'static str,
    pub pu_type: &'static str,
    pub version: u32,
}

/// The universal interface for any native hardware component.
pub trait Pu: Send + Sync {
    /// Return identification metadata.
    fn identify(&self) -> PuInfo;

    /// Read a value from a hardware register at the given offset.
    fn read_reg(&self, offset: usize) -> u32;

    /// Write a value to a hardware register at the given offset.
    fn write_reg(&self, offset: usize, value: u32);

    /// Read a block of data from storage.
    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), crate::hal::HalError> {
        Err(crate::hal::HalError::StorageFailure)
    }

    /// Write a block of data to storage.
    fn write_block(&self, _lba: u64, _data: &[u8]) -> Result<(), crate::hal::HalError> {
        Err(crate::hal::HalError::StorageFailure)
    }

    /// Flush storage caches.
    fn flush(&self) -> Result<(), crate::hal::HalError> {
        Err(crate::hal::HalError::StorageFailure)
    }
}

/// The global registry of indexed Processing Units.
pub struct PuRegistry {
    pockets: Mutex<BTreeMap<PuId, Box<dyn Pu>>>,
}

impl PuRegistry {
    pub const fn new() -> Self {
        Self {
            pockets: Mutex::new(BTreeMap::new()),
        }
    }

    /// Register a new PU into the system.
    pub fn register(&self, id: PuId, pu: Box<dyn Pu>) {
        self.pockets.lock().insert(id, pu);
    }

    /// Get a value from a PU register.
    pub fn read(&self, id: PuId, offset: usize) -> u32 {
        let lock = self.pockets.lock();
        if let Some(pu) = lock.get(&id) {
            pu.read_reg(offset)
        } else {
            0 // Default for missing hardware
        }
    }

    /// Write a value to a PU register.
    pub fn write(&self, id: PuId, offset: usize, value: u32) {
        let lock = self.pockets.lock();
        if let Some(pu) = lock.get(&id) {
            pu.write_reg(offset, value);
        }
    }

    /// Read a block from a storage PU.
    pub fn read_block(&self, id: PuId, lba: u64, buf: &mut [u8]) -> Result<(), crate::hal::HalError> {
        let lock = self.pockets.lock();
        if let Some(pu) = lock.get(&id) {
            pu.read_block(lba, buf)
        } else {
            Err(crate::hal::HalError::StorageFailure)
        }
    }

    /// Write a block to a storage PU.
    pub fn write_block(&self, id: PuId, lba: u64, data: &[u8]) -> Result<(), crate::hal::HalError> {
        let lock = self.pockets.lock();
        if let Some(pu) = lock.get(&id) {
            pu.write_block(lba, data)
        } else {
            Err(crate::hal::HalError::StorageFailure)
        }
    }

    /// Flush a storage PU.
    pub fn flush(&self, id: PuId) -> Result<(), crate::hal::HalError> {
        let lock = self.pockets.lock();
        if let Some(pu) = lock.get(&id) {
            pu.flush()
        } else {
            Err(crate::hal::HalError::StorageFailure)
        }
    }

    /// List all currently indexed PUs.
    pub fn list_pushed(&self) -> Vec<PuId> {
        let lock = self.pockets.lock();
        lock.keys().cloned().collect()
    }
}

lazy_static! {
    pub static ref REGISTRY: PuRegistry = PuRegistry::new();
}

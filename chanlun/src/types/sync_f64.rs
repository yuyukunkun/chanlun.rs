use std::sync::atomic::{AtomicU64, Ordering};

/// f64 原子类型 — 基于 AtomicU64 + 位转换，API 与 `Cell<f64>` 一致。
#[derive(Debug, Default)]
pub struct SyncF64(AtomicU64);

impl SyncF64 {
    pub fn new(v: f64) -> Self {
        Self(AtomicU64::new(v.to_bits()))
    }

    pub fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn set(&self, v: f64) {
        self.0.store(v.to_bits(), Ordering::Relaxed);
    }
}

impl Clone for SyncF64 {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

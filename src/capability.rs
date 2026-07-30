use core::marker::PhantomData;

use crate::env::{EnvSnapshot, PhaseKey};

pub trait Right {
    const ENV_KEY: PhaseKey;
    const LEGACY_ENV_KEY: Option<PhaseKey> = None;
}

macro_rules! declare_right {
    ($name:ident, $key:literal, $legacy:literal) => {
        pub enum $name {}

        impl Right for $name {
            const ENV_KEY: PhaseKey = PhaseKey::from_bytes($key.as_bytes());
            const LEGACY_ENV_KEY: Option<PhaseKey> = Some(PhaseKey::from_bytes($legacy.as_bytes()));
        }
    };
}

declare_right!(FabricRight, "ARACH_CAP_FABRIC", "SISYPHUS_CAP_FABRIC");
declare_right!(
    ResonanceRight,
    "ARACH_CAP_RESONANCE",
    "SISYPHUS_CAP_RESONANCE"
);
declare_right!(
    SchedulerRight,
    "ARACH_CAP_SCHEDULER",
    "SISYPHUS_CAP_SCHEDULER"
);
declare_right!(LearningRight, "ARACH_CAP_LEARNING", "SISYPHUS_CAP_LEARNING");
declare_right!(DmaRight, "ARACH_CAP_DMA", "SISYPHUS_CAP_DMA");
declare_right!(
    DeviceMemoryRight,
    "ARACH_CAP_DEVICE_MEMORY",
    "SISYPHUS_CAP_DEVICE_MEMORY"
);

#[repr(transparent)]
pub struct CapHandle<R: Right> {
    raw: u64,
    _right: PhantomData<R>,
}

impl<R: Right> Copy for CapHandle<R> {}

impl<R: Right> Clone for CapHandle<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Right> CapHandle<R> {
    pub const fn as_raw(self) -> u64 {
        self.raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityError {
    Missing,
    Malformed,
    NullHandle,
    Overflow,
}

pub struct CapabilityEnvelope<'environment> {
    environment: &'environment EnvSnapshot,
}

impl<'environment> CapabilityEnvelope<'environment> {
    pub const fn new(environment: &'environment EnvSnapshot) -> Self {
        Self { environment }
    }

    pub fn recv<R: Right>(&self) -> Result<CapHandle<R>, CapabilityError> {
        let bytes = self
            .environment
            .get(R::ENV_KEY)
            .or_else(|| R::LEGACY_ENV_KEY.and_then(|legacy| self.environment.get(legacy)));
        let bytes = bytes.ok_or(CapabilityError::Missing)?;

        let raw = parse_handle(bytes)?;
        if raw == 0 {
            return Err(CapabilityError::NullHandle);
        }

        Ok(CapHandle {
            raw,
            _right: PhantomData,
        })
    }
}

fn parse_handle(bytes: &[u8]) -> Result<u64, CapabilityError> {
    let (radix, digits) = if bytes.starts_with(b"0x") || bytes.starts_with(b"0X") {
        (16_u64, &bytes[2..])
    } else {
        (10_u64, bytes)
    };

    if digits.is_empty() {
        return Err(CapabilityError::Malformed);
    }

    let mut value = 0_u64;
    let mut consumed = 0_usize;

    for byte in digits.iter().copied() {
        if byte == b'_' {
            continue;
        }

        let digit = match byte {
            b'0'..=b'9' => u64::from(byte - b'0'),
            b'a'..=b'f' if radix == 16 => u64::from(byte - b'a' + 10),
            b'A'..=b'F' if radix == 16 => u64::from(byte - b'A' + 10),
            _ => return Err(CapabilityError::Malformed),
        };

        if digit >= radix {
            return Err(CapabilityError::Malformed);
        }

        value = value
            .checked_mul(radix)
            .and_then(|value| value.checked_add(digit))
            .ok_or(CapabilityError::Overflow)?;

        consumed += 1;
    }

    if consumed == 0 {
        return Err(CapabilityError::Malformed);
    }

    Ok(value)
}

//! Native Crest pointing-device ABI.
//!
//! Boulder normalizes one trusted controller packet at a time.  This client
//! exposes neither raw controller bytes nor an I/O capability to user space.

use crate::SyscallError;
#[cfg(target_os = "none")]
use crate::syscall;
use crate::syscalls::SYS_INPUT_NEXT;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerMotion {
    pub delta_x: i16,
    pub delta_y: i16,
    pub buttons: u8,
    _reserved: [u8; 3],
}

impl PointerMotion {
    pub const fn new(delta_x: i16, delta_y: i16, buttons: u8) -> Self {
        Self {
            delta_x,
            delta_y,
            buttons,
            _reserved: [0; 3],
        }
    }
}

/// Returns at most one authentic pointer motion packet.  `None` is normal
/// when the controller has no packet waiting; it is not a synthetic event.
#[cfg(target_os = "none")]
pub fn next_pointer_motion() -> Result<Option<PointerMotion>, SyscallError> {
    let mut motion = PointerMotion::new(0, 0, 0);
    let available = unsafe {
        syscall(
            SYS_INPUT_NEXT,
            [
                (&mut motion as *mut PointerMotion) as usize,
                core::mem::size_of::<PointerMotion>(),
                0,
                0,
                0,
                0,
            ],
        )?
    };
    match available {
        0 => Ok(None),
        1 => Ok(Some(motion)),
        _ => Err(SyscallError(-74)),
    }
}

#[cfg(not(target_os = "none"))]
pub fn next_pointer_motion() -> Result<Option<PointerMotion>, SyscallError> {
    let _ = SYS_INPUT_NEXT;
    Err(SyscallError(-38))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_motion_has_a_stable_wire_size() {
        assert_eq!(core::mem::size_of::<PointerMotion>(), 8);
        assert_eq!(PointerMotion::new(-4, 9, 1).buttons, 1);
    }
}

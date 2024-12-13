// SPDX-License-Identifier: GPL-2.0

//! Block device abstraction.
//!
//! C header: [`include/linux/blk_types.h`](srctree/include/linux/blk_types.h)
//! C header: [`include/linux/blkdev.h`](srctree/include/linux/blkdev.h)

use kernel::{
    types::ARef,
    error::{self, Error, Result},
    str::CStr,
    device,
    impl_device_type,
    c_str,
};


impl_device_type!(
    BlockDeviceType,
    bd_device,
    kernel::bindings::block_device,
    |device: *const bindings::device| {
	// SAFETY: TODO
        unsafe { CStr::from_char_ptr((*(*device).class).name).as_bytes() == "block".as_bytes() }
    }
);

pub type BlockDevice = device::TypedDevice<BlockDeviceType>;

/// A block device (`struct block_device`).
///
/// # Invariants
///
/// As it wraps a `Device`, which is reference counted,
/// the underlying `struct block_device` remains valid for the lifetime
/// of the instance.
impl BlockDevice {

    pub fn from_path(pathname: &CStr)  -> Result<ARef<Self>> {
        let mut device = core::mem::MaybeUninit::<bindings::dev_t>::uninit();
	// SAFETY: TODO
        let status = unsafe{ bindings::lookup_bdev(pathname.as_char_ptr(), device.as_mut_ptr()) };
	if status != 0 {
	    return Err(Error::from_errno(status));
	}
	// SAFETY: TODO
        let device = unsafe{ device.assume_init() };
        device::Device::from_devt(c_str!("block"), device)?.try_into().or(Err(error::code::EINVAL))
    }
    pub fn capacity_sectors(&self) -> bindings::sector_t {
        self.as_raw_ref().bd_nr_sectors
    }
    pub fn logical_block_size(&self) -> ffi::c_uint {
	// SAFETY: TODO
        unsafe { (*self.as_raw_ref().bd_queue).limits.logical_block_size }
    }
    pub fn physical_block_size(&self) -> ffi::c_uint {
	// SAFETY: TODO
        unsafe { (*self.as_raw_ref().bd_queue).limits.physical_block_size }
    }
}

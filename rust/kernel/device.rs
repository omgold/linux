// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](srctree/include/linux/device.h)

use crate::{
    bindings,
    error::{self, Result},
    str::CStr,
    types::{ARef, AlwaysRefCounted, Opaque, RefCounted},
    subsys::get_subsys_by_class_name,
};
use core::{fmt, ptr};

#[cfg(CONFIG_PRINTK)]
use crate::c_str;
use core::ffi;
use core::ptr::NonNull;

/// A reference-counted device.
///
/// This structure represents the Rust abstraction for a C `struct device`. This implementation
/// abstracts the usage of an already existing C `struct device` within Rust code that we get
/// passed from the C side.
///
/// An instance of this abstraction can be obtained temporarily or permanent.
///
/// A temporary one is bound to the lifetime of the C `struct device` pointer used for creation.
/// A permanent instance is always reference-counted and hence not restricted by any lifetime
/// boundaries.
///
/// For subsystems it is recommended to create a permanent instance to wrap into a subsystem
/// specific device structure (e.g. `pci::Device`). This is useful for passing it to drivers in
/// `T::probe()`, such that a driver can store the `ARef<Device>` (equivalent to storing a
/// `struct device` pointer in a C driver) for arbitrary purposes, e.g. allocating DMA coherent
/// memory.
///
/// # Invariants
///
/// A `Device` instance represents a valid `struct device` created by the C portion of the kernel.
///
/// Instances of this type are always reference-counted, that is, a call to `get_device` ensures
/// that the allocation remains valid at least until the matching call to `put_device`.
///
/// `bindings::device::release` is valid to be called from any thread, hence `ARef<Device>` can be
/// dropped from any thread.
#[repr(transparent)]
pub struct Device(Opaque<bindings::device>);

impl Device {
    /// Creates a new reference-counted abstraction instance of an existing `struct device` pointer.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count,
    /// i.e. it must be ensured that the reference count of the C `struct device` `ptr` points to
    /// can't drop to zero, for the duration of this function call.
    ///
    /// It must also be ensured that `bindings::device::release` can be called from any thread.
    /// While not officially documented, this should be the case for any `struct device`.
    pub unsafe fn get_device(ptr: *mut bindings::device) -> ARef<Self> {
        // SAFETY: By the safety requirements ptr is valid
        unsafe { Self::as_ref(ptr) }.into()
    }

    pub fn from_major_minor(class: &CStr, major: ffi::c_uint, minor: ffi::c_uint) -> Result<ARef<Self>> {
        let device: bindings::dev_t = major << bindings::MINORBITS | minor;
        Self::from_devt(class, device)
    }

    pub fn from_devt(class: &CStr, device: bindings::dev_t) -> Result<ARef<Self>> {
        let subsys =
	    // SAFETY: TODO
            get_subsys_by_class_name(class)?;
	// SAFETY: TODO
        let device = unsafe {
            bindings::class_find_device(
                subsys.class(),
                ptr::null(),
                &device as *const _ as *const _,
                Some(bindings::device_match_devt),
            )
        };
	// SAFETY: TODO
        Ok(unsafe { Self::get_device(device) })
    }

    pub fn from_name(class: &CStr, name: &CStr) -> Result<ARef<Self>> {
        let subsys =
            // SAFETY: TODO
            get_subsys_by_class_name(class)?;
	// SAFETY: TODO
        let device = unsafe {
            bindings::class_find_device(
                subsys.class(),
                ptr::null(),
                name.as_ptr() as *const _,
                Some(bindings::device_match_name),
            )
        };
	// SAFETY: TODO
        Ok(unsafe { Self::get_device(device) })
    }

    /// Obtain the raw `struct device *`.
    pub(crate) fn as_raw(&self) -> *mut bindings::device {
        self.0.get()
    }

    /// Convert a raw C `struct device` pointer to a `&'a Device`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count,
    /// i.e. it must be ensured that the reference count of the C `struct device` `ptr` points to
    /// can't drop to zero, for the duration of this function call and the entire duration when the
    /// returned reference exists.
    pub unsafe fn as_ref<'a>(ptr: *mut bindings::device) -> &'a Self {
        // SAFETY: Guaranteed by the safety requirements of the function.
        unsafe { &*ptr.cast() }
    }

    /// Prints an emergency-level message (level 0) prefixed with device information.
    ///
    /// More details are available from [`dev_emerg`].
    ///
    /// [`dev_emerg`]: crate::dev_emerg
    pub fn pr_emerg(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_EMERG, args) };
    }

    /// Prints an alert-level message (level 1) prefixed with device information.
    ///
    /// More details are available from [`dev_alert`].
    ///
    /// [`dev_alert`]: crate::dev_alert
    pub fn pr_alert(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_ALERT, args) };
    }

    /// Prints a critical-level message (level 2) prefixed with device information.
    ///
    /// More details are available from [`dev_crit`].
    ///
    /// [`dev_crit`]: crate::dev_crit
    pub fn pr_crit(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_CRIT, args) };
    }

    /// Prints an error-level message (level 3) prefixed with device information.
    ///
    /// More details are available from [`dev_err`].
    ///
    /// [`dev_err`]: crate::dev_err
    pub fn pr_err(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_ERR, args) };
    }

    /// Prints a warning-level message (level 4) prefixed with device information.
    ///
    /// More details are available from [`dev_warn`].
    ///
    /// [`dev_warn`]: crate::dev_warn
    pub fn pr_warn(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_WARNING, args) };
    }

    /// Prints a notice-level message (level 5) prefixed with device information.
    ///
    /// More details are available from [`dev_notice`].
    ///
    /// [`dev_notice`]: crate::dev_notice
    pub fn pr_notice(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_NOTICE, args) };
    }

    /// Prints an info-level message (level 6) prefixed with device information.
    ///
    /// More details are available from [`dev_info`].
    ///
    /// [`dev_info`]: crate::dev_info
    pub fn pr_info(&self, args: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
        unsafe { self.printk(bindings::KERN_INFO, args) };
    }

    /// Prints a debug-level message (level 7) prefixed with device information.
    ///
    /// More details are available from [`dev_dbg`].
    ///
    /// [`dev_dbg`]: crate::dev_dbg
    pub fn pr_dbg(&self, args: fmt::Arguments<'_>) {
        if cfg!(debug_assertions) {
            // SAFETY: `klevel` is null-terminated, uses one of the kernel constants.
            unsafe { self.printk(bindings::KERN_DEBUG, args) };
        }
    }

    /// Prints the provided message to the console.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `klevel` is null-terminated; in particular, one of the
    /// `KERN_*`constants, for example, `KERN_CRIT`, `KERN_ALERT`, etc.
    #[cfg_attr(not(CONFIG_PRINTK), allow(unused_variables))]
    unsafe fn printk(&self, klevel: &[u8], msg: fmt::Arguments<'_>) {
        // SAFETY: `klevel` is null-terminated and one of the kernel constants. `self.as_raw`
        // is valid because `self` is valid. The "%pA" format string expects a pointer to
        // `fmt::Arguments`, which is what we're passing as the last argument.
        #[cfg(CONFIG_PRINTK)]
        unsafe {
            bindings::_dev_printk(
                klevel as *const _ as *const crate::ffi::c_char,
                self.as_raw(),
                c_str!("%pA").as_char_ptr(),
                &msg as *const _ as *const crate::ffi::c_void,
            )
        };
    }

    /// Checks if property is present or not.
    pub fn property_present(&self, name: &CStr) -> bool {
        // SAFETY: By the invariant of `CStr`, `name` is null-terminated.
        unsafe { bindings::device_property_present(self.as_raw().cast_const(), name.as_char_ptr()) }
    }
}

// SAFETY: Instances of `Device` are always reference-counted.
unsafe impl RefCounted for Device {
    fn inc_ref(&self) {
        // SAFETY: The existence of a shared reference guarantees that the refcount is non-zero.
        unsafe { bindings::get_device(self.as_raw()) };
    }

    unsafe fn dec_ref(obj: ptr::NonNull<Self>) {
        // SAFETY: The safety requirements guarantee that the refcount is non-zero.
        unsafe { bindings::put_device(obj.cast().as_ptr()) }
    }
}

// SAFETY: We do not implement `Ownable`, thus it is okay to can obtain an `Device<Task>` from a
// `&Device`.
unsafe impl AlwaysRefCounted for Device {}

// SAFETY: As by the type invariant `Device` can be sent to any thread.
unsafe impl Send for Device {}

// SAFETY: `Device` can be shared among threads because all immutable methods are protected by the
// synchronization in `struct device`.
unsafe impl Sync for Device {}

#[doc(hidden)]
#[macro_export]
macro_rules! dev_printk {
    ($method:ident, $dev:expr, $($f:tt)*) => {
        {
            ($dev).$method(core::format_args!($($f)*));
        }
    }
}

/// Prints an emergency-level message (level 0) prefixed with device information.
///
/// This level should be used if the system is unusable.
///
/// Equivalent to the kernel's `dev_emerg` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_emerg!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_emerg {
    ($($f:tt)*) => { $crate::dev_printk!(pr_emerg, $($f)*); }
}

/// Prints an alert-level message (level 1) prefixed with device information.
///
/// This level should be used if action must be taken immediately.
///
/// Equivalent to the kernel's `dev_alert` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_alert!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_alert {
    ($($f:tt)*) => { $crate::dev_printk!(pr_alert, $($f)*); }
}

/// Prints a critical-level message (level 2) prefixed with device information.
///
/// This level should be used in critical conditions.
///
/// Equivalent to the kernel's `dev_crit` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_crit!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_crit {
    ($($f:tt)*) => { $crate::dev_printk!(pr_crit, $($f)*); }
}

/// Prints an error-level message (level 3) prefixed with device information.
///
/// This level should be used in error conditions.
///
/// Equivalent to the kernel's `dev_err` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_err!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_err {
    ($($f:tt)*) => { $crate::dev_printk!(pr_err, $($f)*); }
}

/// Prints a warning-level message (level 4) prefixed with device information.
///
/// This level should be used in warning conditions.
///
/// Equivalent to the kernel's `dev_warn` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_warn!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_warn {
    ($($f:tt)*) => { $crate::dev_printk!(pr_warn, $($f)*); }
}

/// Prints a notice-level message (level 5) prefixed with device information.
///
/// This level should be used in normal but significant conditions.
///
/// Equivalent to the kernel's `dev_notice` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_notice!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_notice {
    ($($f:tt)*) => { $crate::dev_printk!(pr_notice, $($f)*); }
}

/// Prints an info-level message (level 6) prefixed with device information.
///
/// This level should be used for informational messages.
///
/// Equivalent to the kernel's `dev_info` macro.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_info!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_info {
    ($($f:tt)*) => { $crate::dev_printk!(pr_info, $($f)*); }
}

/// Prints a debug-level message (level 7) prefixed with device information.
///
/// This level should be used for debug messages.
///
/// Equivalent to the kernel's `dev_dbg` macro, except that it doesn't support dynamic debug yet.
///
/// Mimics the interface of [`std::print!`]. More information about the syntax is available from
/// [`core::fmt`] and `alloc::format!`.
///
/// [`std::print!`]: https://doc.rust-lang.org/std/macro.print.html
///
/// # Examples
///
/// ```
/// # use kernel::device::Device;
///
/// fn example(dev: &Device) {
///     dev_dbg!(dev, "hello {}\n", "there");
/// }
/// ```
#[macro_export]
macro_rules! dev_dbg {
    ($($f:tt)*) => { $crate::dev_printk!(pr_dbg, $($f)*); }
}

pub trait DeviceType {
    type RawType;
    unsafe fn as_generic(device: *const Self::RawType) -> NonNull<bindings::device>;
    unsafe fn as_typed(device: *const bindings::device) -> NonNull<Self::RawType>;
    unsafe fn is_type(device: *const bindings::device) -> bool;
}

/// A typed device (`struct typed_device`).
#[repr(transparent)]
pub struct TypedDevice<T: DeviceType>(Opaque<T::RawType>);

impl<T: DeviceType> TypedDevice<T> {

    pub fn from_major_minor(class: &CStr, major: ffi::c_uint, minor: ffi::c_uint) -> Result<ARef<Self>> {
        Device::from_major_minor(class, major, minor)?.try_into().or(Err(error::code::EINVAL))
    }

    pub fn from_devt(class: &CStr, device: bindings::dev_t) -> Result<ARef<Self>> {
        Device::from_devt(class, device)?.try_into().or(Err(error::code::EINVAL))
    }

    pub fn from_name(class: &CStr, name: &CStr) -> Result<ARef<Self>> {
        Device::from_name(class, name)?.try_into().or(Err(error::code::EINVAL))
    }

    unsafe fn from_generic_ptr(device: *mut bindings::device) -> Result<ARef<Self>> {
        match NonNull::new(device) {
            None => Err(error::code::ENODEV),
	    // SAFETY: TODO
            Some(device) => match unsafe { T::is_type(device.as_ptr()) } {
		// SAFETY: TODO
                true => Ok(unsafe { Self::from_raw(T::as_typed(device.as_ptr()).as_ptr()) }),
                false => Err(error::code::EINVAL),
            },
        }
    }

    unsafe fn from_raw(device: *mut T::RawType) -> ARef<Self> {
	// SAFETY: TODO
        unsafe { Self::as_ref(device) }.into()
    }

    pub(crate) fn as_raw(&self) -> *mut T::RawType {
        self.0.get()
    }

    pub(crate) fn as_raw_ref(&self) -> &T::RawType {
	// SAFETY: TODO
        unsafe { &*self.0.get() }
    }

    /// Convert a raw C `struct device` pointer to a `&'a Device`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count,
    /// i.e. it must be ensured that the reference count of the C `struct device` `ptr` points to
    /// can't drop to zero, for the duration of this function call and the entire duration when the
    /// returned reference exists.
    unsafe fn as_ref<'a>(ptr: *mut T::RawType) -> &'a Self {
        // SAFETY: Guaranteed by the safety requirements of the function.
        unsafe { &*ptr.cast() }
    }
}

// SAFETY: Instances of `Device` are reference-counted.
unsafe impl<T: DeviceType> RefCounted for TypedDevice<T> {
    fn inc_ref(&self) {
        // SAFETY: The existence of a shared reference guarantees that the refcount is non-zero.
        unsafe { bindings::get_device(T::as_generic(self.as_raw()).as_ptr()) };
    }

    unsafe fn dec_ref(obj: NonNull<Self>) {
        // SAFETY: The safety requirements guarantee that the refcount is non-zero.
        unsafe { bindings::put_device(T::as_generic(obj.as_ref().as_raw()).as_ptr()) }
    }
}

// SAFETY: TODO
unsafe impl<T: DeviceType> AlwaysRefCounted for TypedDevice<T> {}

// SAFETY: As by the type invariant `TypedDevice` can be sent to any thread.
unsafe impl<T: DeviceType> Send for TypedDevice<T> {}

// SAFETY: `Device` can be shared among threads because all immutable methods are protected by the
// synchronization in `struct device`.
unsafe impl<T: DeviceType> Sync for TypedDevice<T> {}

impl<T: DeviceType> From<ARef<TypedDevice<T>>> for ARef<Device> {
    fn from(this: ARef<TypedDevice<T>>) -> Self {
	// SAFETY: TODO
        unsafe {
	    Device::get_device(T::as_generic(ARef::into_raw(this).as_ref().as_raw()).as_ptr())
	}
    }
}

impl<T: DeviceType> From<&TypedDevice<T>> for &Device {
    fn from(this: &TypedDevice<T>) -> Self {
	// SAFETY: TODO
        unsafe { Device::as_ref(T::as_generic(this.as_raw()).as_ptr()) }
    }
}

impl<C: DeviceType> TryFrom<ARef<Device>> for ARef<TypedDevice<C>> {
    type Error = ARef<Device>;
    fn try_from(device: ARef<Device>) -> Result<Self, ARef<Device>> {
        let device = device.as_raw();
	// SAFETY: TODO
        match unsafe { TypedDevice::<C>::from_generic_ptr(device) } {
            Ok(device) => Ok(device),
	    // SAFETY: TODO
            Err(_) => Err(unsafe { Device::get_device(device) }),
        }
    }
}

/// Helper to implement a [`DeviceType`]
///
/// Example:
/// ```
/// use kernel::impl_device_type;
///
/// impl_device_type!(BlockDeviceType, bd_device, kernel::bindings::block_device,
///     |device: *const kernel::bindings::device| {
///         unsafe { &kernel::bindings::block_class as *const _ == (*device).class }
///     }
/// );
/// ```
#[macro_export]
macro_rules! impl_device_type {
    ($name:ident, $field:ident, $raw_type:path, $is_type:expr) => {
        pub struct $name;

        impl $crate::device::DeviceType for $name {
            type RawType = $raw_type;
            unsafe fn as_typed(
                device: *const $crate::bindings::device,
            ) -> core::ptr::NonNull<Self::RawType> {
		// SAFETY: TODO
                unsafe {
                    core::ptr::NonNull::new_unchecked($crate::container_of!(
                        device, $raw_type, $field
                    ) as *mut Self::RawType)
                }
            }
            unsafe fn as_generic(
                device: *const Self::RawType,
            ) -> core::ptr::NonNull<$crate::bindings::device> {
		// SAFETY: TODO
                unsafe {
                    core::ptr::NonNull::new_unchecked(
                        &raw const (*device).$field as *mut _,
                    )
                }
            }
            unsafe fn is_type(device: *const $crate::bindings::device) -> bool {
                $is_type(device)
            }
        }
    };
}

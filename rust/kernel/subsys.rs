use crate::{
    bindings, container_of,
    error::{self, Result},
    str::CStr,
    types::{ARef, RefCounted, Opaque},
};

use core::ptr::NonNull;

#[repr(transparent)]
pub struct SubsysPrivate(Opaque<bindings::subsys_private>);

// SAFETY: TODO
unsafe impl RefCounted for SubsysPrivate {
    fn inc_ref(&self) {
	// SAFETY: TODO
        unsafe { bindings::subsys_get(self.0.get()) };
    }
    unsafe fn dec_ref(obj: NonNull<Self>) {
	// SAFETY: TODO
        unsafe { bindings::subsys_put(obj.as_ref().0.get()) };
    }
}

impl SubsysPrivate {
    pub unsafe fn from_raw(this: NonNull<bindings::subsys_private>) -> ARef<Self> {
	// SAFETY: TODO
        unsafe { ARef::from_raw(this.cast()) }
    }
    pub fn class<'a>(&'a self) -> &'a bindings::class {
	// SAFETY: TODO
        unsafe { &*(*self.0.get()).class }
    }
}

pub fn get_subsys_by_class_name(name: &CStr) -> Result<ARef<SubsysPrivate>> {
    // SAFETY: TODO
    let class_kset = unsafe {
        NonNull::new(bindings::get_class_kset())
            .ok_or(error::code::ENOENT)?
            .as_ptr()
    };
    // SAFETY: TODO
    match unsafe { NonNull::new(bindings::kset_find_obj(class_kset, name.as_ptr())) } {
        Some(kobj) => {
	    // SAFETY: TODO
            let kset = unsafe { container_of!(kobj.as_ptr(), bindings::kset, kobj) };
	    // SAFETY: TODO
            let sp = unsafe { container_of!(kset, bindings::subsys_private, subsys) };
	    // SAFETY: TODO
            Ok(unsafe { SubsysPrivate::from_raw(NonNull::new_unchecked(sp as _)) })
        }
        None => Err(error::code::ENOENT),
    }
}

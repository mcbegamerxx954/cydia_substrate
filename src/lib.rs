use std::{
    ffi::{CStr, CString},
    ptr,
};

use libc::c_void;
use objc2::{
    ffi::{objc_class, IMP},
    runtime::{AnyClass, Imp, Sel},
};

// A mobile substrate "image" in memory
// This will close the "image" on drop
// "image" : native os module or whatever idk
// but not a visual type image
pub struct NativeImagePtr(*mut libc::c_void);
impl NativeImagePtr {
    pub fn from_filename(filename: &str) -> Self {
        let cstr = CString::new(filename).unwrap();
        let raw = unsafe { substrate_sys::MSMapImage(cstr.as_ptr()) };
        Self(raw)
    }

    pub fn from_name(name: &str) -> Self {
        let cstr = CString::new(name).unwrap();
        let raw = unsafe { substrate_sys::MSGetImageByName(cstr.as_ptr()) };
        Self(raw)
    }

    pub fn find_symbol(&self, symname: &str) -> *mut c_void {
        let cstr = CString::new(symname).unwrap();
        unsafe { substrate_sys::MSFindSymbol(self.0, cstr.as_ptr()) }
    }
    pub fn from_address(&self, addr: usize) -> String {
        // waaa
        let mut ptr = addr as *mut c_void;
        let ptr_ptr: *mut *mut c_void = &mut ptr;
        unsafe {
            let raw_cstr = substrate_sys::MSFindAddress(self.0, ptr_ptr);
            let cstr = CStr::from_ptr(raw_cstr);
            cstr.to_str().unwrap().to_owned()
        }
    }
}
impl Drop for NativeImagePtr {
    // Make it so that when it drops it releases
    // the actual MSImage object in memory
    fn drop(&mut self) {
        unsafe { substrate_sys::MSCloseImage(self.0) }
    }
}

// this problem is very hard to tackle for my brain
// so i will leave it to the caller (last words)
pub unsafe fn hook_function(orig: *mut c_void, hook: *mut c_void) -> *const c_void {
    let mut old: *mut c_void = ptr::null_mut();
    let old_ptr: *mut *mut c_void = &mut old;
    substrate_sys::MSHookFunction(orig, hook, old_ptr);
    *old_ptr
}
/// Hook some bytes in memory, this writes as many bytes as your data has
///
/// # Safety: You should know what youre doing.
pub unsafe fn hook_memory(target: *mut c_void, data: &[u8]) {
    let data_size = data.len();
    let data_ptr = data.as_ptr();
    substrate_sys::MSHookMemory(target, data_ptr.cast(), data_size)
}
/// Hook a objc Message
///
/// # Safety: You must ensure that you handle the returned imp properly
/// and convert it to the fn type you wanna call
pub unsafe fn hook_message(class: &AnyClass, sel: Sel, imp: Imp) -> IMP {
    // Assuming that AnyClass is a zst this should be safe
    let c_class: *const AnyClass = class;
    let c_class: *const objc_class = c_class.cast();
    // preparing pointer to fn pointer which
    // substrate will store the old fn to
    let mut old: *mut c_void = ptr::null_mut();
    let old_ptr: *mut *mut c_void = &mut old;
    // do the thing
    substrate_sys::MSHookMessageEx(c_class, sel.as_ptr(), Some(imp), old_ptr);
    if old.is_null() {
        None
    } else {
        // Should be safe since it isnt null
        Some(std::mem::transmute(old))
    }
}

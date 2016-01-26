extern crate libc;
extern crate pocketsphinx_sys as bindings;

use std::ptr;
use std::ffi::{CStr, CString};
use std::mem;
use libc::c_char;

pub struct CmdLn {
    raw: *mut bindings::cmd_ln_t,

    // Holds argument strings because Sphinx doesn't copy them and
    // assumes they are valid as long as `cmd_ln_t` is alive.
    #[allow(dead_code)]
    args: Vec<CString>,
}

impl CmdLn {
    pub fn init(strict: bool, args: &[&str]) -> Option<Self> {
        // Sphinx assumes that `args` are valid as long as returned
        // `cmd_ln_t` is alive, so copy them.
        let c_args: Vec<_> = args.iter().map(|s| CString::new(*s).unwrap()).collect();
        let args_ptrs: Vec<_> = c_args.iter().map(|s| s.as_ptr()).collect();
        let raw = unsafe {
            bindings::cmd_ln_parse_r(ptr::null_mut(),
                                     bindings::ps_args(),
                                     args_ptrs.len() as i32,
                                     args_ptrs.as_ptr(),
                                     strict as i32)
        };
        if raw.is_null() {
            return None;
        }
        Some(CmdLn{raw: raw, args: c_args})
    }

    fn into_raw(mut self) -> *mut bindings::cmd_ln_t {
        mem::replace(&mut self.raw, ptr::null_mut())
    }
}

impl Drop for CmdLn {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            let ref_count = unsafe { bindings::cmd_ln_free_r(self.raw) };
            assert!(ref_count == 0);
        }
    }
}

pub struct PsDecoder {
    raw: *mut bindings::ps_decoder_t,
}

impl PsDecoder {
    pub fn init(config: CmdLn) -> Self {
        let raw = unsafe { bindings::ps_init(config.into_raw()) };
        assert!(!raw.is_null());
        PsDecoder{raw: raw}
    }

    pub fn start_utt(&self, utt_id: Option<&str>) -> std::result::Result<(), ()>  {
        let (_id_cstr, id_ptr) = utt_id.map_or_else(
            ||  { (CString::new("").unwrap(), ptr::null()) },
            |s| {
                let cstr = CString::new(s).unwrap();
                let cptr = cstr.as_ptr();
                (cstr, cptr)
            }
        );
        let code = unsafe { bindings::ps_start_utt(self.raw, id_ptr) };
        if code == 0 { Ok(()) } else { Err(()) }
    }

    pub fn process_raw(&self,
                       data: &[i16],
                       no_search: bool,
                       full_utt: bool) -> std::result::Result<i32, ()> {
        let frames = unsafe {
            bindings::ps_process_raw(self.raw, data.as_ptr(), data.len(),
                                     no_search as i32, full_utt as i32)
        };
        if frames < 0 { return Err(()); }
        Ok(frames)
    }

    pub fn end_utt(&self) -> std::result::Result<(), ()> {
        let code = unsafe { bindings::ps_end_utt(self.raw) };
        if code < 0 { return Err(()); }
        Ok(())
    }

    pub fn get_hyp(&self) -> Option<(String, String, i32)> {
        let mut score: i32 = 0;
        let mut c_utt_id: *const c_char = ptr::null();
        let c_hyp = unsafe { bindings::ps_get_hyp(self.raw, &mut score, &mut c_utt_id) };
        if c_hyp.is_null() { return None; }

        let hyp = unsafe { CStr::from_ptr(c_hyp) }.to_string_lossy().into_owned();
        let utt_id = unsafe { CStr::from_ptr(c_utt_id) }.to_string_lossy().into_owned();
        Some((hyp, utt_id, score))
    }
}

impl Drop for PsDecoder {
    fn drop(&mut self) {
        let ref_count = unsafe { bindings::ps_free(self.raw) };
        assert!(ref_count == 0);
    }
}

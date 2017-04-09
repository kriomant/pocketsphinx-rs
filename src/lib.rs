extern crate libc;
extern crate pocketsphinx_sys as bindings;

use std::ptr;
use std::ffi::{CStr, CString};
use libc::c_char;

pub use search::*;
pub use nbest::*;
pub use jsgf::*;
pub use error::*;

mod search;
mod nbest;
mod jsgf;
mod error;

pub struct CmdLn {
    raw: *mut bindings::cmd_ln_t,
}

impl CmdLn {
    pub fn init(strict: bool, args: &[&str]) -> Result<Self> {
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
            return Err(Error);
        }
        Ok(CmdLn{raw: raw})
    }

    pub unsafe fn get_str(&self, name: &str) -> &str {
        let c_str = bindings::cmd_ln_str_r(self.raw, CString::new(name).unwrap().as_ptr());
        CStr::from_ptr(c_str).to_str().unwrap()
    }

    pub unsafe fn get_int(&self, name: &str) -> i64 {
        bindings::cmd_ln_int_r(self.raw, CString::new(name).unwrap().as_ptr())
    }

    pub unsafe fn get_float(&self, name: &str) -> f64 {
        bindings::cmd_ln_float_r(self.raw, CString::new(name).unwrap().as_ptr())
    }

    pub fn exists(&self, name: &str) -> bool {
        let res = unsafe { bindings::cmd_ln_exists_r(self.raw, CString::new(name).unwrap().as_ptr()) };
        res != 0
    }

    pub unsafe fn get_boolean(&self, name: &str) -> bool {
        self.get_int(name) != 0
    }

    pub unsafe fn get_int32(&self, name: &str) -> i32 {
        self.get_int(name) as i32
    }

    pub unsafe fn get_float32(&self, name: &str) -> f32 {
        self.get_float(name) as f32
    }

    pub unsafe fn get_float64(&self, name: &str) -> f64 {
        self.get_float(name) as f64
    }
}

impl Drop for CmdLn {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { bindings::cmd_ln_free_r(self.raw) };
        }
    }
}


pub struct PsDecoder {
    raw: *mut bindings::ps_decoder_t,
}

impl PsDecoder {
    pub fn init(config: CmdLn) -> Self {
        let raw = unsafe { bindings::ps_init(config.raw) };
        assert!(!raw.is_null());
        PsDecoder{raw: raw}
    }

    pub fn start_utt(&self, utt_id: Option<&str>) -> Result<()>  {
        let (_id_cstr, id_ptr) = utt_id.map_or_else(
            ||  { (CString::new("").unwrap(), ptr::null()) },
            |s| {
                let cstr = CString::new(s).unwrap();
                let cptr = cstr.as_ptr();
                (cstr, cptr)
            }
        );
        let code = unsafe { bindings::ps_start_utt(self.raw, id_ptr) };
        if code == 0 { Ok(()) } else { Err(Error) }
    }

    pub fn process_raw(&self,
                       data: &[i16],
                       no_search: bool,
                       full_utt: bool) -> Result<i32> {
        let frames = unsafe {
            bindings::ps_process_raw(self.raw, data.as_ptr(), data.len(),
                                     no_search as i32, full_utt as i32)
        };
        if frames < 0 { return Err(Error); }
        Ok(frames)
    }

    pub fn end_utt(&self) -> Result<()> {
        let code = unsafe { bindings::ps_end_utt(self.raw) };
        if code < 0 { return Err(Error); }
        Ok(())
    }

    pub fn get_hyp(&self) -> Option<(String, Option<String>, i32)> {
        let mut score: i32 = 0;
        let mut c_utt_id: *const c_char = ptr::null();
        let c_hyp = unsafe { bindings::ps_get_hyp(self.raw, &mut score, &mut c_utt_id) };
        if c_hyp.is_null() { return None; }

        let hyp = unsafe { CStr::from_ptr(c_hyp) }.to_string_lossy().into_owned();
        let utt_id = if c_utt_id.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(c_utt_id) }.to_string_lossy().into_owned())
        };
        Some((hyp, utt_id, score))
    }

    pub fn get_in_speech(&self) -> bool {
        let res = unsafe { bindings::ps_get_in_speech(self.raw) };
        res == 1
    }

    pub fn get_prob(&self) -> i32 {
        unsafe { bindings::ps_get_prob(self.raw) }
    }

    pub fn get_n_frames(&self) -> i32 {
        unsafe { bindings::ps_get_n_frames(self.raw) }
    }

    pub fn nbest(&self, start_frame: i32, end_frame: i32,
                 ctx1: Option<&str>, ctx2: Option<&str>) -> NBestIter {
        let c_ctx1 = ctx1.map(|s| CString::new(s).unwrap());
        let c_ctx2 = ctx2.map(|s| CString::new(s).unwrap());
        let raw_nbest = unsafe {
            bindings::ps_nbest(self.raw, start_frame, end_frame,
                               c_ctx1.map_or(ptr::null(), |c| c.as_ptr()),
                               c_ctx2.map_or(ptr::null(), |c| c.as_ptr()))
        };
        NBestIter::new(raw_nbest)
    }

    pub fn nbest_simple(&self) -> NBestIter {
        self.nbest(0, -1, None, None)
    }

    pub fn seg_iter(&self) -> SegIter {
        let mut best_score: i32 = 0;
        SegIter::new(unsafe { bindings::ps_seg_iter(self.raw, &mut best_score) })
    }

    pub fn searches(&self) -> Searches {
        Searches::new(self.raw)
    }
}

impl Drop for PsDecoder {
    fn drop(&mut self) {
        let ref_count = unsafe { bindings::ps_free(self.raw) };
        assert!(ref_count == 0);
    }
}

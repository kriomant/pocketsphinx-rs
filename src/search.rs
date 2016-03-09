use bindings;

use std;
use std::ffi::{CStr, CString, OsStr};
use libc::c_int;

use std::os::unix::ffi::OsStrExt;

use super::PsDecoder;
use super::{Error, Result};

pub struct Searches<'a> {
    raw: *mut bindings::ps_search_iter_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Searches<'a> {
    pub fn new(raw_ps: *const bindings::ps_decoder_t) -> Self {
        let iter = unsafe { bindings::ps_search_iter(raw_ps) };
        Searches { raw: iter, _marker: std::marker::PhantomData }
    }
}

impl<'a> Drop for Searches<'a> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { bindings::ps_search_iter_free(self.raw) };
        }
    }
}

impl<'a> Iterator for Searches<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_null() {
            None
        } else {
            let item = unsafe { bindings::ps_search_iter_val(self.raw) };
            self.raw = unsafe { bindings::ps_search_iter_next(self.raw) };
            Some(unsafe { CStr::from_ptr(item).to_str().unwrap() })
        }
    }
}

pub trait PsDecoderSearchExt {
    fn set_search(&mut self, name: &str) -> Result<()>;
    fn get_search(&self) -> Option<&str>;
    fn unset_search(&mut self, name: &str) -> Result<()>;
    fn set_lm_file(&mut self, name: &str, path: &OsStr) -> Result<()>;
    fn set_jsgf_file(&mut self, name: &str, path: &OsStr) -> Result<()>;
    fn set_jsgf_string(&mut self, name: &str, jsgf_string: &str) -> Result<()>;
    fn ps_set_kws(&mut self, name: &str, keyfile: &OsStr) -> Result<()>;
    fn set_keyphrase(&mut self, name: &str, keyphrase: &str) -> Result<()>;
    fn set_allphone_file(&mut self, name: &str, path: &OsStr) -> Result<()>;
}

fn check_res(res: c_int) -> Result<()> {
    match res {
        0 => Ok(()),
        -1 => Err(Error),
        _ => unreachable!(),
    }
}

impl PsDecoderSearchExt for PsDecoder {
    fn set_search(&mut self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        check_res(unsafe { bindings::ps_set_search(self.raw, name_cstr.as_ptr()) })
    }

    fn get_search(&self) -> Option<&str> {
        let name_c = unsafe { bindings::ps_get_search(self.raw) };
        if name_c.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(name_c) }.to_str().unwrap())
        }
    }

    fn unset_search(&mut self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        check_res(unsafe { bindings::ps_unset_search(self.raw, name_cstr.as_ptr()) })
    }

    fn set_lm_file(&mut self, name: &str, path: &OsStr) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let path_c = CString::new(path.as_bytes()).unwrap();
        check_res(unsafe { bindings::ps_set_lm_file(self.raw, name_c.as_ptr(), path_c.as_ptr()) })
    }

    fn set_jsgf_file(&mut self, name: &str, path: &OsStr) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let path_c = CString::new(path.as_bytes()).unwrap();
        check_res(unsafe {
            bindings::ps_set_jsgf_file(self.raw, name_c.as_ptr(), path_c.as_ptr())
        })
    }

    fn set_jsgf_string(&mut self, name: &str, jsgf_string: &str) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let jsgf_string_c = CString::new(jsgf_string).unwrap();
        check_res(unsafe {
            bindings::ps_set_jsgf_string(self.raw, name_c.as_ptr(), jsgf_string_c.as_ptr())
        })
    }

    fn ps_set_kws(&mut self, name: &str, keyfile: &OsStr) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let keyfile_c = CString::new(keyfile.as_bytes()).unwrap();
        check_res(unsafe {
            bindings::ps_set_kws(self.raw, name_c.as_ptr(), keyfile_c.as_ptr())
        })
    }
    fn set_keyphrase(&mut self, name: &str, keyphrase: &str) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let keyphrase_c = CString::new(keyphrase).unwrap();
        check_res(unsafe {
            bindings::ps_set_keyphrase(self.raw, name_c.as_ptr(), keyphrase_c.as_ptr())
        })
    }

    fn set_allphone_file(&mut self, name: &str, path: &OsStr) -> Result<()> {
        let name_c = CString::new(name).unwrap();
        let path_c = CString::new(path.as_bytes()).unwrap();
        check_res(unsafe {
            bindings::ps_set_allphone_file(self.raw, name_c.as_ptr(), path_c.as_ptr())
        })
    }
}

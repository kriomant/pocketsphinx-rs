use bindings;

use std;
use std::ptr;
use std::ffi::{CStr, CString, OsStr};

use std::os::unix::ffi::OsStrExt;

use super::{Error, Result};

pub mod internal {
    use std;
    use bindings;
    use libc::c_char;
    use std::ffi::CStr;

    #[derive(Clone)]
    pub struct Tags<'a> {
        node: bindings::glist_t,
        _marker: std::marker::PhantomData<&'a str>,
    }

    impl<'a> Tags<'a> {
        pub fn new(node: bindings::glist_t) -> Self {
            Tags { node: node, _marker: std::marker::PhantomData }
        }
    }

    impl<'a> Iterator for Tags<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            if self.node.is_null() {
                return None
            }

            let tag = unsafe { CStr::from_ptr((*self.node).data.as_ptr() as *const c_char) };
            self.node = unsafe { (*self.node).next };
            Some(tag.to_str().unwrap())
        }
    }

    #[derive(Clone)]
    pub struct Atom<'a> {
        raw: *const bindings::internal::jsgf_atom_t,
        _marker: std::marker::PhantomData<&'a str>,
    }

    impl<'a> Atom<'a> {
        pub fn new(raw: *const bindings::internal::jsgf_atom_t) -> Self {
            Atom { raw: raw, _marker: std::marker::PhantomData }
        }

        pub fn name(&self) -> &'a str {
            unsafe { CStr::from_ptr((*self.raw).name).to_str().unwrap() }
        }

        pub fn tags(&self) -> Tags<'a>  {
            Tags::new(unsafe { (*self.raw).tags })
        }
    }

    #[derive(Clone)]
    pub struct Atoms<'a> {
        node: bindings::glist_t,
        _marker: std::marker::PhantomData<&'a str>,
    }

    impl<'a> Atoms<'a> {
        fn new(node: bindings::glist_t) -> Self {
            Atoms { node: node, _marker: std::marker::PhantomData }
        }
    }

    impl<'a> Iterator for Atoms<'a> {
        type Item = Atom<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.node.is_null() {
                return None
            }

            let atom = Atom::new(unsafe {
                (*self.node).data.as_ptr() as *const bindings::internal::jsgf_atom_t
            });
            self.node = unsafe { (*self.node).next };
            Some(atom)
        }
    }

    #[derive(Clone)]
    pub struct Alternatives<'a> {
        node: *const bindings::internal::jsgf_rhs_t,
        _marker: std::marker::PhantomData<&'a str>,
    }

    impl<'a> Alternatives<'a> {
        fn new(node: *const bindings::internal::jsgf_rhs_t) -> Self {
            Alternatives { node: node, _marker: std::marker::PhantomData }
        }
    }

    impl<'a> Iterator for Alternatives<'a> {
        type Item = Atoms<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.node.is_null() {
                return None;
            }

            let atoms = Atoms::new(unsafe { (*self.node).atoms });
            self.node = unsafe { (*self.node).alt };
            Some(atoms)
        }
    }

    #[derive(Clone)]
    pub struct RuleData<'a> {
        raw: *const bindings::internal::jsgf_rule_s,
        _marker: std::marker::PhantomData<&'a super::Jsgf>,
    }

    impl<'a> RuleData<'a> {
        pub fn new(raw: *const bindings::internal::jsgf_rule_s) -> Self {
            RuleData { raw: raw, _marker: std::marker::PhantomData }
        }

        pub fn alternatives(&self) -> Alternatives<'a> {
            Alternatives::new(unsafe { (*self.raw).rhs })
        }
    }
}

#[derive(Clone)]
pub struct Rule<'a> {
    raw: *const bindings::jsgf_rule_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Rule<'a> {
    fn new(raw: *const bindings::jsgf_rule_t) -> Self {
        Rule { raw: raw, _marker: std::marker::PhantomData }
    }

    pub fn name(&self) -> &'a str {
        let name_c = unsafe { bindings::jsgf_rule_name(self.raw) };
        unsafe { CStr::from_ptr(name_c) }.to_str().unwrap()
    }

    pub fn is_public(&self) -> bool {
        let res = unsafe { bindings::jsgf_rule_public(self.raw) };
        res != 0
    }

    pub unsafe fn internal(&self) -> internal::RuleData<'a> {
        internal::RuleData::new(self.raw as *const bindings::internal::jsgf_rule_s)
    }
}

pub struct Rules<'a> {
    raw: *const bindings::jsgf_rule_iter_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Rules<'a> {
    pub fn new(raw: *const bindings::jsgf_rule_iter_t) -> Self {
        Rules { raw: raw, _marker: std::marker::PhantomData }
    }
}

impl<'a> Drop for Rules<'a> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { bindings::jsgf_rule_iter_free(self.raw) };
        }
    }
}

impl<'a> Iterator for Rules<'a> {
    type Item = Rule<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_null() {
            None
        } else {
            let item = Rule::new(unsafe { bindings::jsgf_rule_iter_rule(self.raw) });
            self.raw = unsafe { bindings::jsgf_rule_iter_next(self.raw) };
            Some(item)
        }
    }
}

pub struct Jsgf {
    raw: *mut bindings::jsgf_t,
}

impl Jsgf {
    pub fn parse_file(filename: &OsStr) -> Result<Self> {
        let filename_c = CString::new(filename.as_bytes()).unwrap();
        let raw = unsafe { bindings::jsgf_parse_file(filename_c.as_ptr(), ptr::null()) };
        if raw.is_null() {
            Err(Error)
        } else {
            Ok(Jsgf { raw: raw })
        }
    }

    pub fn parse_string(s: &str) -> Result<Self> {
        let s_c = CString::new(s.as_bytes()).unwrap();
        let raw = unsafe { bindings::jsgf_parse_string(s_c.as_ptr(), ptr::null()) };
        if raw.is_null() {
            Err(Error)
        } else {
            Ok(Jsgf { raw: raw })
        }
    }

    pub fn name(&self) -> &str {
        let name_c = unsafe { bindings::jsgf_grammar_name(self.raw) };
        assert!(!name_c.is_null());
        unsafe { CStr::from_ptr(name_c) }.to_str().unwrap()
    }

    pub fn rules(&self) -> Rules {
        let raw_rules_iter = unsafe { bindings::jsgf_rule_iter(self.raw) };
        assert!(!raw_rules_iter.is_null());
        Rules::new(raw_rules_iter)
    }

    pub fn public_rule(&self) -> Rule {
        Rule::new(unsafe { bindings::jsgf_get_public_rule(self.raw) })
    }

    pub fn rule<'a>(&'a self, name: &str) -> Option<Rule<'a>> {
        let name_c = CString::new(name).unwrap();
        let raw_rule = unsafe { bindings::jsgf_get_rule(self.raw, name_c.as_ptr()) };
        if raw_rule.is_null() {
            None
        } else {
            Some(Rule::new(raw_rule))
        }
    }
}

impl Drop for Jsgf {
    fn drop(&mut self) {
        unsafe { bindings::jsgf_grammar_free(self.raw) }
    }
}

use bindings;

use std;
use std::ffi::CStr;

pub struct SegProbs {
    pub prob: i32,
    pub ascr: i32,
    pub lscr: i32,
    pub lback: i32,
}

pub struct Seg<'a> {
    raw: *const bindings::ps_seg_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Seg<'a> {
    fn new(raw: *const bindings::ps_seg_t) -> Self {
        Seg { raw: raw, _marker: std::marker::PhantomData }
    }

    pub fn prob(&self) -> SegProbs {
        let mut probs = SegProbs { prob: 0, ascr: 0, lscr: 0, lback: 0 };
        probs.prob = unsafe { bindings::ps_seg_prob(self.raw, &mut probs.ascr,
                                                    &mut probs.lscr, &mut probs.lback) };
        probs
    }

    pub fn word(&self) -> &str {
        let c_word = unsafe { bindings::ps_seg_word(self.raw) };
        unsafe { CStr::from_ptr(c_word) }.to_str().unwrap()
    }

    pub fn frames(&self) -> (i32, i32) {
        let mut sf: i32 = 0;
        let mut ef: i32 = 0;
        unsafe { bindings::ps_seg_frames(self.raw, &mut sf, &mut ef) }
        (sf, ef)
    }
}

pub struct SegIter<'a> {
    raw: *mut bindings::ps_seg_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> SegIter<'a> {
    pub fn new(raw: *mut bindings::ps_seg_t) -> Self {
        SegIter { raw: raw, _marker: std::marker::PhantomData }
    }
}

impl<'a> Drop for SegIter<'a> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { bindings::ps_seg_free(self.raw) };
        }
    }
}

impl<'a> Iterator for SegIter<'a> {
    type Item = Seg<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_null() {
            None
        } else {
            let item = Seg::new(self.raw);
            self.raw = unsafe { bindings::ps_seg_next(self.raw) };
            Some(item)
        }
    }
}

pub struct NBest<'a> {
    raw: *const bindings::ps_nbest_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> NBest<'a> {
    fn new(raw: *const bindings::ps_nbest_t) -> Self {
        NBest { raw: raw, _marker: std::marker::PhantomData }
    }

    pub fn hyp(&self) -> (&str, i32) {
        let mut score: i32 = 0;
        let c_hyp = unsafe { bindings::ps_nbest_hyp(self.raw, &mut score) };
        if c_hyp.is_null() {
            ("", 0)
        } else {
            (unsafe { CStr::from_ptr(c_hyp) }.to_str().unwrap(), score)
        }
    }

    pub fn segments(&self) -> (SegIter, i32) {
        let mut score: i32 = 0;
        let seg_raw = unsafe { bindings::ps_nbest_seg(self.raw, &mut score) };
        (SegIter::new(seg_raw), score)
    }
}

pub struct NBestIter<'a> {
    raw: *mut bindings::ps_nbest_t,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> NBestIter<'a> {
    pub fn new(raw: *mut bindings::ps_nbest_t) -> Self {
        NBestIter { raw: raw, _marker: std::marker::PhantomData }
    }
}

impl<'a> Drop for NBestIter<'a> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { bindings::ps_nbest_free(self.raw) };
        }
    }
}

impl<'a> Iterator for NBestIter<'a> {
    type Item = NBest<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.raw.is_null() {
            None
        } else {
            let item = NBest::new(self.raw);
            self.raw = unsafe { bindings::ps_nbest_next(self.raw) };
            Some(item)
        }
    }
}

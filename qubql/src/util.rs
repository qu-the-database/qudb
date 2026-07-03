use std::{borrow::Borrow, cell::{RefCell, RefMut}, collections::VecDeque, fmt, hash::Hash, mem::transmute, ops::{Deref, DerefMut}, rc::Rc};

#[derive(Clone, Eq)]
pub enum Str<'a> {
    Borrowed(&'a str),
    Owned(Box<str>),
}
impl<'a> Str<'a> {
    pub const fn borrowed<'b>(&'b self) -> Str<'b> where 'a: 'b {
        match self {
            Str::Owned(x) => Str::Borrowed(&**x),
            Str::Borrowed(x) => Str::Borrowed(*x),
        }
    }
}
impl<'a, 'b> PartialEq<Str<'b>> for Str<'a> {
    fn eq(&self, other: &Str<'b>) -> bool {
        AsRef::<str>::as_ref(self) == AsRef::<str>::as_ref(other)
    }
}
impl<'a> PartialEq<str> for Str<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
impl<'a, 'b> PartialEq<&'b str> for Str<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        self.as_ref() == *other
    }
}
impl<'a> Hash for Str<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}
impl<'a> Borrow<str> for Str<'a> {
    fn borrow(&self) -> &str {
        match self {
            Str::Borrowed(x) => x,
            Str::Owned(x) => x,
        }
    }
}
impl<'a> AsRef<str> for Str<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Str::Borrowed(x) => x,
            Str::Owned(x) => x,
        }
    }
}
impl<'a> Deref for Str<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Str::Borrowed(x) => x,
            Str::Owned(x) => x,
        }
    }
}
impl<'a> fmt::Display for Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl<'a> fmt::Debug for Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

pub struct LockedItem<'a, T, I: Iterator<Item = T>> {
    #[allow(unused)] re: RefMut<'a, LockedData<T, I>>,
    ptr: *mut T,
}
impl<'a, T, I: Iterator<Item = T>> Deref for LockedItem<'a, T, I> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: thanks to the lock, this pointer is always valid.
        unsafe {
            self.ptr.as_ref().unwrap_unchecked()
        }
    }
}
impl<'a, T, I: Iterator<Item = T>> DerefMut for LockedItem<'a, T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: thanks to the lock, this pointer is always valid.
        unsafe {
            self.ptr.as_mut().unwrap_unchecked()
        }
    }
}
impl<T: fmt::Debug, I: Iterator<Item = T>> fmt::Debug for LockedItem<'_, T, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}
impl<T: fmt::Display, I: Iterator<Item = T>> fmt::Display for LockedItem<'_, T, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}
impl<T: PartialEq, I: Iterator<Item = T>> PartialEq<T> for LockedItem<'_, T, I> {
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other)
    }

    fn ne(&self, other: &T) -> bool {
        self.deref().ne(other)
    }
}
struct LockedData<T, I: Iterator<Item = T>> {
    it: I,
    data: VecDeque<T>,
    start: usize,
    locks: Vec<(usize, usize)>,
}
impl<T, I: Iterator<Item = T>> LockedData<T, I> {
    fn recalc_data(&mut self) {
        if let Some(pos) = self.locks.iter().map(|x| x.0).min() {
            for _ in self.start..pos {
                self.data.pop_front();
            }
            self.start = pos;
        }
    }
}
pub struct LockedIter<T, I: Iterator<Item = T>> {
    cache: Rc<RefCell<LockedData<T, I>>>,
    progress: usize,
}
impl<T, I: Iterator<Item = T>> LockedIter<T, I> {
    pub fn new(iter: I) -> Self {
        Self {
            cache: Rc::new(RefCell::new(LockedData {
                it: iter,
                data: VecDeque::new(),
                start: 0,
                locks: vec![(0, 1)],
            })),
            progress: 0,
        }
    }

    pub const fn as_mut(&mut self) -> &mut Self { self }
}
// Rust doesn't let me do it better, so here we go.
impl<'a, T, I: Iterator<Item = T>> Iterator for &'a mut LockedIter<T, I> {
    type Item = LockedItem<'a, T, I>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut v = self.cache.borrow_mut();
        let i = self.progress - v.start;
        let ret = match v.data.get_mut(i) {
            None => {
                match v.it.next() {
                    Some(x) => {
                        let i = v.data.len();
                        v.data.push_back(x);
                        let ptr = v.data.get_mut(i).unwrap() as *mut T;
                        Some(ptr)
                    }
                    None => None,
                }
            },
            Some(x) => {
                let ptr = x as *mut T;
                Some(ptr)
            }
        };
        if ret.is_some() {
            let lock = v.locks.iter_mut().find(|x| x.0 == self.progress).unwrap();
            if lock.1 == 1 {
                lock.0 += 1;
            } else {
                lock.1 -= 1;
                v.locks.push((self.progress + 1, 1));
            }
            self.progress += 1;
            v.recalc_data();
        }
        ret.map(|ptr| unsafe { LockedItem { re: transmute(v), ptr } })
    }
}
impl<T, I: Iterator<Item = T>> Clone for LockedIter<T, I> {
    fn clone(&self) -> Self {
        let mut v = self.cache.borrow_mut();
        v.locks.iter_mut().find(|x| x.0 == self.progress).unwrap().1 += 1;
        Self {
            cache: self.cache.clone(),
            progress: self.progress,
        }
    }
}
impl<T, I: Iterator<Item = T>> Drop for LockedIter<T, I> {
    fn drop(&mut self) {
        let mut v = self.cache.borrow_mut();
        let (i, x) = v.locks.iter_mut().enumerate().find(|x| x.1.0 == self.progress).unwrap();
        x.1 -= 1;
        if x.1 == 0 {
            v.locks.remove(i);
        }
        v.recalc_data();
    }
}
impl<T, I: Iterator<Item = T>> fmt::Debug for LockedIter<T, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("LockedIter {{ progress = {} }}", self.progress))
    }
}

#[cfg(test)]
mod tests {
    mod locked_iter {
        use std::ops::Deref;

        use crate::util::LockedIter;

        #[test]
        fn basic_iteration() {
            let vec = [1, 2, 3, 4, 5];
            let mut iter = LockedIter::new(vec.into_iter());

            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(1));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(2));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(3));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(4));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(5));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), None);
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), None);
        }

        #[test]
        fn locked_iteration() {
            let vec = [1, 2, 3, 4, 5];
            let mut iter = LockedIter::new(vec.into_iter());

            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(1));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(2));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(3));

            let mut iter2 = iter.clone();

            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(4));
            assert_eq!(iter2.as_mut().next().map(|x| *x.deref()), Some(4));
            assert_eq!(iter2.as_mut().next().map(|x| *x.deref()), Some(5));
            assert_eq!(iter2.as_mut().next().map(|x| *x.deref()), None);
            assert_eq!(iter2.as_mut().next().map(|x| *x.deref()), None);

            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), Some(5));
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), None);
            assert_eq!(iter.as_mut().next().map(|x| *x.deref()), None);
        }
    }
}

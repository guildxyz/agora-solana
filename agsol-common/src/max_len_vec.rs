use super::{MaxLenResult, MaxSerializedLen, CONTENTS_FULL};
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;

// NOTE anyhow doesn't compile under bpf it seems

#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct MaxLenVec<T, const N: usize> {
    contents: Vec<T>,
}

impl<T, const N: usize> MaxSerializedLen for MaxLenVec<T, N>
where
    T: MaxSerializedLen,
{
    const MAX_SERIALIZED_LEN: usize = 4 + N * T::MAX_SERIALIZED_LEN;
}

impl<T, const N: usize> MaxLenVec<T, N> {
    pub fn new() -> Self {
        MaxLenVec {
            contents: Vec::with_capacity(N),
        }
    }

    pub fn is_full(&self) -> bool {
        self.contents.len() == N
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }

    pub fn contents(&self) -> &[T] {
        self.contents.as_slice()
    }

    pub fn contents_mut(&mut self) -> &mut [T] {
        self.contents.as_mut_slice()
    }

    pub fn push(&mut self, elem: T) -> MaxLenResult {
        if self.is_full() {
            Err(CONTENTS_FULL)
        } else {
            self.contents.push(elem);
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.contents.pop()
    }

    pub fn cyclic_push(&mut self, elem: T) {
        if self.is_full() {
            self.contents.remove(0);
        }
        self.contents.push(elem);
    }

    pub fn insert(&mut self, index: usize, value: T) -> MaxLenResult {
        if self.is_full() {
            Err(CONTENTS_FULL)
        } else {
            self.contents.insert(index, value);
            Ok(())
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.contents.remove(index);
    }

    pub fn get_last_element(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.contents[self.contents.len() - 1])
        }
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for MaxLenVec<T, N> {
    type Error = &'static str;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        if vec.len() > N {
            Err(CONTENTS_FULL)
        } else {
            Ok(Self { contents: vec })
        }
    }
}

impl<T, const N: usize> Default for MaxLenVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test_max_len_vec {
    use super::*;

    const CAPACITY: usize = 5;
    type TestVec = MaxLenVec<u8, CAPACITY>;

    #[test]
    fn initialization() {
        assert_eq!(TestVec::new().contents.capacity(), CAPACITY);
        let vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        assert!(TestVec::try_from(vec).is_ok());
        let long_vec: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        assert!(TestVec::try_from(long_vec).is_err());
    }

    #[test]
    fn dynamic_updates() {
        let mut vec = TestVec::new();
        assert_eq!(vec.get_last_element(), None);
        for i in 0..CAPACITY {
            assert!(vec.push(i as u8).is_ok());
        }
        assert_eq!(vec.len(), CAPACITY);
        assert!(vec.push(32).is_err());
        vec.cyclic_push(32);
        assert_eq!(vec.contents(), &[1, 2, 3, 4, 32]);
        assert_eq!(vec.get_last_element(), Some(&32));
        vec.pop();
        vec.pop();
        assert_eq!(vec.contents(), &[1, 2, 3]);
        vec.cyclic_push(53);
        assert_eq!(vec.contents(), &[1, 2, 3, 53]);
        vec.cyclic_push(23);
        vec.cyclic_push(33);
        vec.cyclic_push(73);
        assert_eq!(vec.contents(), &[3, 53, 23, 33, 73]);
        assert_eq!(vec.get_last_element(), Some(&73));
        assert!(vec.insert(3, 12).is_err());
        vec.pop();
        assert!(vec.insert(3, 12).is_ok());
        assert_eq!(vec.contents(), &[3, 53, 23, 12, 33]);
        vec.remove(2);
        assert_eq!(vec.contents(), &[3, 53, 12, 33]);
        vec.remove(1);
        assert_eq!(vec.contents(), &[3, 12, 33]);
        vec.remove(0);
        assert_eq!(vec.contents(), &[12, 33]);
        vec.remove(1);
        assert_eq!(vec.contents(), &[12]);
        vec.pop();
        assert!(vec.is_empty());
    }

    #[test]
    fn static_updates() {
        let mut vec = TestVec::try_from(vec![3, 5, 2, 1, 4]).unwrap();
        vec.contents_mut().sort_unstable();
        assert_eq!(vec.contents(), &[1, 2, 3, 4, 5]);
        vec.contents_mut()[2] = 10;
        assert_eq!(vec.contents(), &[1, 2, 10, 4, 5]);
    }

    #[test]
    fn max_len_vec_serialized_len() {
        let mut test_vec = TestVec::new();
        assert!(test_vec.try_to_vec().unwrap().len() <= TestVec::MAX_SERIALIZED_LEN);

        for i in 0..4 {
            assert!(test_vec.push(i).is_ok());
        }
        assert!(test_vec.try_to_vec().unwrap().len() <= TestVec::MAX_SERIALIZED_LEN);

        assert!(test_vec.push(4).is_ok());
        assert_eq!(
            test_vec.try_to_vec().unwrap().len(),
            TestVec::MAX_SERIALIZED_LEN
        );
    }
}

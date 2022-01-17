use super::{MaxSerializedLen, CONTENTS_FULL};

use borsh::{BorshDeserialize, BorshSerialize};

use std::convert::TryFrom;

#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct MaxLenString<const N: usize> {
    contents: String,
}

impl<const N: usize> MaxSerializedLen for MaxLenString<N> {
    const MAX_SERIALIZED_LEN: usize = 4 + N;
}

impl<const N: usize> TryFrom<String> for MaxLenString<N> {
    type Error = &'static str;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        if string.as_bytes().len() > N {
            return Err(CONTENTS_FULL);
        }
        Ok(Self { contents: string })
    }
}

impl<const N: usize> TryFrom<&str> for MaxLenString<N> {
    type Error = &'static str;

    fn try_from(string_slice: &str) -> Result<Self, Self::Error> {
        Self::try_from(string_slice.to_owned())
    }
}

#[cfg(test)]
mod test_max_len_string {
    use super::*;

    type TestString = MaxLenString<5>;

    #[test]
    fn valid_conversions() {
        let string_slice = "ASDEF";
        let max_len_string = TestString::try_from(string_slice).unwrap();
        assert_eq!(string_slice, max_len_string.contents);

        let string = "ASDEF".to_string();
        let max_len_string = TestString::try_from(string.clone()).unwrap();
        assert_eq!(string, max_len_string.contents);
    }

    #[test]
    fn invalid_conversions() {
        let string_slice = "ASDEFG";
        assert!(TestString::try_from(string_slice).is_err());

        let string = "ASDEFG".to_string();
        assert!(TestString::try_from(string).is_err());
    }

    #[test]
    fn max_len_string_serialized_len() {
        let test_string: TestString = TestString::try_from("asd").unwrap();
        assert!(test_string.try_to_vec().unwrap().len() <= TestString::MAX_SERIALIZED_LEN);

        let test_string: TestString = TestString::try_from("asdef".to_string()).unwrap();
        assert_eq!(
            test_string.try_to_vec().unwrap().len(),
            TestString::MAX_SERIALIZED_LEN
        );
    }
}

use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

/// Trait that provides the maximum length of the serialized byte stream of a
/// borsh-serializable data structure.
///
/// Useful when allocating space for a Solana account upon creation.
///
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate agsol_common_derive;
/// use agsol_common::MaxSerializedLen;
/// use borsh::{BorshSerialize, BorshDeserialize};
/// use solana_program::pubkey::Pubkey;
///
/// #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen)]
/// struct FooStruct {
///     foo: u64, // max len: 8
///     bar: i32, // max len: 4
/// }
///
/// #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen)]
/// struct BarStruct {
///     foo: [u8; 32], // max len: 32
///     #[len(4 + 8 * 2)]
///     bar: Vec<u16>, // max len: 20
///     baz: Option<FooStruct>, // max len: 13
/// }
///
/// #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen)]
/// enum FooEnum {
///     Foo { // max len: 40 + 1
///         a: u64,
///         b: Pubkey,
///     },
///     Bar, // max len: 1
///     Baz(Option<Pubkey>), // max len: 1 + 1 + 32
///     #[len(200)]
///     Quux(String), // max len: 1 + 200
/// }
///
/// # fn main() {
/// assert_eq!(FooStruct::MAX_SERIALIZED_LEN, 12);
/// assert_eq!(BarStruct::MAX_SERIALIZED_LEN, 65);
/// assert_eq!(FooEnum::MAX_SERIALIZED_LEN, 201);
/// # }
/// ```
///
/// # Notes
/// Note, that for enums with more than ~15 variants, the compiler hangs due to
/// the way the maximum lengths of the variants are computed. Therefore, it is
/// not recommended to derive `MaxSerializedLen` for enums like that, rather it
/// should be implemented manually.
pub trait MaxSerializedLen {
    const MAX_SERIALIZED_LEN: usize;
}

macro_rules! impl_max_serialized_length {
    ($this:ty, $len:expr) => {
        impl MaxSerializedLen for $this {
            const MAX_SERIALIZED_LEN: usize = $len;
        }
    };
}

impl_max_serialized_length!(bool, 1);
impl_max_serialized_length!(u8, 1);
impl_max_serialized_length!(u16, 2);
impl_max_serialized_length!(u32, 4);
impl_max_serialized_length!(u64, 8);
impl_max_serialized_length!(u128, 16);
impl_max_serialized_length!(i8, 1);
impl_max_serialized_length!(i16, 2);
impl_max_serialized_length!(i32, 4);
impl_max_serialized_length!(i64, 8);
impl_max_serialized_length!(i128, 16);
impl_max_serialized_length!(Pubkey, 32);
impl_max_serialized_length!([u8; 32], 32);

impl<T> MaxSerializedLen for Option<T>
where
    T: MaxSerializedLen,
{
    const MAX_SERIALIZED_LEN: usize = 1 + T::MAX_SERIALIZED_LEN;
}

impl<T> MaxSerializedLen for PhantomData<T> {
    const MAX_SERIALIZED_LEN: usize = 0;
}

#[cfg(test)]
mod test {
    use super::*;
    use borsh::{BorshSerialize, BorshDeserialize};
    use solana_program::clock::UnixTimestamp;

    #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen, Debug)]
    struct Something {
        a: u64,
        b: i32,
    }

    #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen, Debug)]
    struct Dummy {
        something: Something,
        #[len(4 + 8 * 2)]
        c: Vec<u16>,
    }

    #[repr(C)]
    #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen, Debug)]
    struct DummyOption {
        a: u64,
        b: Option<Dummy>,
    }

    #[test]
    fn test_derive() {
        assert_eq!(Dummy::MAX_SERIALIZED_LEN, 32);
    }

    #[test]
    fn serialized_lenghts() {
        let u: UnixTimestamp = 234232;
        assert_eq!(
            u.try_to_vec().unwrap().len(),
            UnixTimestamp::MAX_SERIALIZED_LEN
        );
    }

    #[test]
    fn option_max_serialized_len() {
        let none: Option<u64> = None;
        assert!(none.try_to_vec().unwrap().len() <= Option::<u64>::MAX_SERIALIZED_LEN);
        let none: Option<u64> = Some(15_u64);
        assert_eq!(
            none.try_to_vec().unwrap().len(),
            Option::<u64>::MAX_SERIALIZED_LEN
        );

        let mut dummy_option = DummyOption {
            a: u64::MAX,
            b: None,
        };
        assert!(dummy_option.try_to_vec().unwrap().len() <= DummyOption::MAX_SERIALIZED_LEN);
        dummy_option.b = Some(Dummy {
            something: Something { a: 0, b: 1456 },
            c: vec![542; 8],
        });
        assert_eq!(
            dummy_option.try_to_vec().unwrap().len(),
            DummyOption::MAX_SERIALIZED_LEN
        );
    }

    #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen, Debug)]
    enum DummyEnum {
        Hello {
            a: u64,
            b: Pubkey,
        },
        Bello,
        Yello(Option<Pubkey>),
        #[len(200)]
        Zello(String),
    }

    #[derive(BorshSerialize, BorshDeserialize, MaxSerializedLen, Debug)]
    enum OtherEnum {
        Dummy(DummyEnum),
        Foo,
        Bar {
            #[len(220)]
            foo: String,
            bar: u8,
        },
        Baz(DummyOption),
    }

    #[test]
    fn enum_max_serialized_len() {
        let en = DummyEnum::Hello {
            a: 89,
            b: Pubkey::new_unique(),
        };
        assert_eq!(DummyEnum::MAX_SERIALIZED_LEN, 201);
        assert_eq!(en.try_to_vec().unwrap().len(), 41);

        let en = OtherEnum::Dummy(en);
        assert_eq!(OtherEnum::MAX_SERIALIZED_LEN, 222);
        assert_eq!(en.try_to_vec().unwrap().len(), 42);

        let en = OtherEnum::Baz(DummyOption {
            a: 1234,
            b: Some(Dummy {
                something: Something { a: 100, b: 200 },
                c: vec![2256; 8],
            }),
        });
        assert_eq!(
            en.try_to_vec().unwrap().len(),
            1 + DummyOption::MAX_SERIALIZED_LEN
        );
    }

    #[derive(MaxSerializedLen, BorshSerialize, BorshDeserialize, Debug)]
    enum DummyUnitEnum {
        Hello,
        Bello,
        Yello,
    }

    #[derive(MaxSerializedLen, BorshSerialize, BorshDeserialize, Debug)]
    struct DummyStructWithArray {
        foo: [u32; 3],
        bar: [Option<Pubkey>; 2],
        baz: DummyUnitEnum,
    }

    #[test]
    fn unit_enum_and_arrays() {
        let mut dummy = DummyStructWithArray {
            foo: [324, 222, 432224],
            bar: [Some(Pubkey::new_unique()), Some(Pubkey::new_unique())],
            baz: DummyUnitEnum::Bello,
        };

        assert_eq!(
            dummy.try_to_vec().unwrap().len(),
            DummyStructWithArray::MAX_SERIALIZED_LEN
        );

        dummy.bar[0] = None;
        assert_eq!(
            dummy.try_to_vec().unwrap().len(),
            DummyStructWithArray::MAX_SERIALIZED_LEN - 32
        );
    }

    #[derive(MaxSerializedLen, BorshDeserialize, BorshSerialize, Debug)]
    struct GhastlyStruct<T> {
        foo: u8,
        bar: Option<Pubkey>,
        baz: PhantomData<T>,
    }

    #[test]
    fn phantom_data() {
        assert_eq!(
            GhastlyStruct::<DummyStructWithArray>::MAX_SERIALIZED_LEN,
            34
        );
        assert_eq!(GhastlyStruct::<DummyUnitEnum>::MAX_SERIALIZED_LEN, 34);
    }
}

use super::MaxSerializedLen;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use std::cmp::Ordering;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct LockedItem<T: BorshDeserialize + BorshSerialize + MaxSerializedLen> {
    pub item: T,
    pub expires: UnixTimestamp,
}

impl<T> MaxSerializedLen for LockedItem<T>
where
    T: BorshSerialize + BorshDeserialize + MaxSerializedLen,
{
    const MAX_SERIALIZED_LEN: usize = T::MAX_SERIALIZED_LEN + UnixTimestamp::MAX_SERIALIZED_LEN;
}

impl<T> PartialOrd for LockedItem<T>
where
    T: BorshSerialize + BorshDeserialize + MaxSerializedLen,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.expires.cmp(&other.expires))
    }
}

impl<T> Ord for LockedItem<T>
where
    T: BorshSerialize + BorshDeserialize + MaxSerializedLen,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.expires.cmp(&other.expires)
    }
}

impl<T> PartialEq for LockedItem<T>
where
    T: BorshSerialize + BorshDeserialize + MaxSerializedLen,
{
    fn eq(&self, other: &Self) -> bool {
        self.expires == other.expires
    }
}

impl<T> Eq for LockedItem<T> where T: BorshSerialize + BorshDeserialize + MaxSerializedLen {}

impl<T> LockedItem<T>
where
    T: BorshSerialize + BorshDeserialize + MaxSerializedLen,
{
    pub fn expired(&self, current_time: UnixTimestamp) -> bool {
        self.expires < current_time
    }
}

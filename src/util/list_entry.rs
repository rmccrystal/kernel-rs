use winapi::_core::marker::PhantomData;

use crate::include::{_LIST_ENTRY, LIST_ENTRY32};

pub struct ListEntryIterator<T, const OFFSET: isize> {
    current_entry: *mut _LIST_ENTRY,
    start_entry: *mut _LIST_ENTRY,
    phantom: PhantomData<T>,
}

impl<T, const OFFSET: isize> ListEntryIterator<T, OFFSET> {
    pub fn new(list: &mut _LIST_ENTRY) -> Self {
        let current_entry = list.Flink;
        let start_entry = list as _;
        Self { current_entry, start_entry, phantom: PhantomData }
    }
}

impl<T, const OFFSET: isize> Iterator for ListEntryIterator<T, OFFSET> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry == self.start_entry {
            return None;
        }

        let result = unsafe { Some(core::ptr::read((self.current_entry as *mut u8).offset(-OFFSET) as *const T)) };

        // inc linked list
        unsafe { self.current_entry = (*self.current_entry).Flink };

        result
    }
}

pub struct ListEntryIterator32<T, const OFFSET: isize> {
    current_entry: *mut LIST_ENTRY32,
    start_entry: *mut LIST_ENTRY32,
    phantom: PhantomData<T>,
}

impl<T, const OFFSET: isize> ListEntryIterator32<T, OFFSET> {
    pub fn new(list: &mut LIST_ENTRY32) -> Self {
        let current_entry = list.Flink;
        let start_entry = list;
        Self { current_entry: current_entry as _, start_entry: start_entry as _, phantom: PhantomData }
    }
}

impl<T, const OFFSET: isize> Iterator for ListEntryIterator32<T, OFFSET> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry == self.start_entry {
            return None;
        }

        let result = unsafe { Some(core::ptr::read((self.current_entry as *mut u8).offset(-OFFSET) as *const T)) };

        // inc linked list
        unsafe { self.current_entry = (*self.current_entry).Flink as _ };

        result
    }
}

//! Fuzz/test helpers for [`MockFlashBase`] that depend on crate-private item APIs.
//!
//! Kept out of `mock_flash.rs` so doctest `include!("mock_flash.rs")` copies stay compilable
//! when the `_test` feature is enabled.

use crate::mock_flash::MockFlashBase;
use crate::{
    NorFlashExt, calculate_page_address, calculate_page_end_address, calculate_page_index,
};

impl<const PAGES: usize, const BYTES_PER_WORD: usize, const PAGE_WORDS: usize>
    MockFlashBase<PAGES, BYTES_PER_WORD, PAGE_WORDS>
{
    /// Get the presence of the item at the given address.
    ///
    /// - If some, the item is there.
    /// - If true, the item is present and fine.
    /// - If false, the item is corrupt or erased.
    pub async fn get_item_presence(&mut self, target_item_address: u32) -> Option<bool> {
        if !Self::FULL_FLASH_RANGE.contains(&target_item_address) {
            return None;
        }

        let mut buf = [0; 1024 * 16];

        let page_index = calculate_page_index::<Self>(Self::FULL_FLASH_RANGE, target_item_address);

        let page_data_start = calculate_page_address::<Self>(Self::FULL_FLASH_RANGE, page_index)
            + Self::WORD_SIZE as u32;
        let page_data_end = calculate_page_end_address::<Self>(Self::FULL_FLASH_RANGE, page_index)
            - Self::WORD_SIZE as u32;

        let mut found_item = None;
        let mut it = crate::item::ItemHeaderIter::new(page_data_start, page_data_end);
        while let (Some(header), item_address) = it.traverse(self, |_, _| false).await.unwrap() {
            let next_item_address = header.next_item_address::<Self>(item_address);

            if (item_address..next_item_address).contains(&target_item_address) {
                let maybe_item = header
                    .read_item(self, &mut buf, item_address, page_data_end)
                    .await
                    .unwrap();

                match maybe_item {
                    crate::item::MaybeItem::Corrupted(_, _)
                    | crate::item::MaybeItem::Erased(_, _) => {
                        found_item.replace(false);
                        break;
                    }
                    crate::item::MaybeItem::Present(_) => {
                        found_item.replace(true);
                        break;
                    }
                }
            }
        }

        found_item
    }
}

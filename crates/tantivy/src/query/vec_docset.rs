#[cfg(test)]
pub use self::vec_docset_impl::VecDocSet;

#[cfg(test)]
mod vec_docset_impl {
    use common::HasLen;

    use crate::docset::{DocSet, TERMINATED};
    use crate::DocId;

    /// A [`DocSet`] backed by a `Vec<DocId>`.
    ///
    /// The vector must contain sorted, strictly increasing doc IDs.
    /// This is used in tests to create doc sets from a list of doc IDs.
    pub struct VecDocSet {
        doc_ids: Vec<DocId>,
        cursor: usize,
    }

    impl From<Vec<DocId>> for VecDocSet {
        fn from(doc_ids: Vec<DocId>) -> VecDocSet {
            assert!(doc_ids.windows(2).all(|w| w[0] < w[1]));
            VecDocSet { doc_ids, cursor: 0 }
        }
    }

    impl DocSet for VecDocSet {
        fn advance(&mut self) -> DocId {
            self.cursor += 1;
            if self.cursor >= self.doc_ids.len() {
                self.cursor = self.doc_ids.len();
                return TERMINATED;
            }
            self.doc()
        }

        fn doc(&self) -> DocId {
            if self.cursor == self.doc_ids.len() {
                return TERMINATED;
            }
            self.doc_ids[self.cursor]
        }

        fn size_hint(&self) -> u32 {
            self.len() as u32
        }
    }

    impl HasLen for VecDocSet {
        fn len(&self) -> usize {
            self.doc_ids.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::docset::{DocSet, COLLECT_BLOCK_BUFFER_LEN, TERMINATED};
    use crate::query::vec_docset::VecDocSet;
    use crate::DocId;

    #[test]
    pub fn test_vec_postings() {
        let doc_ids: Vec<DocId> = (0u32..1024u32).map(|e| e * 3).collect();
        let mut postings = VecDocSet::from(doc_ids);
        assert_eq!(postings.doc(), 0u32);
        assert_eq!(postings.advance(), 3u32);
        assert_eq!(postings.doc(), 3u32);
        assert_eq!(postings.seek(14u32), 15u32);
        assert_eq!(postings.doc(), 15u32);
        assert_eq!(postings.seek(300u32), 300u32);
        assert_eq!(postings.doc(), 300u32);
        assert_eq!(postings.seek(6000u32), TERMINATED);
    }

    #[test]
    pub fn test_fill_buffer() {
        let doc_ids: Vec<DocId> = (1u32..=(COLLECT_BLOCK_BUFFER_LEN as u32 * 2 + 9)).collect();
        let mut postings = VecDocSet::from(doc_ids);
        let mut buffer = [0u32; COLLECT_BLOCK_BUFFER_LEN];
        assert_eq!(postings.fill_buffer(&mut buffer), COLLECT_BLOCK_BUFFER_LEN);
        for i in 0u32..COLLECT_BLOCK_BUFFER_LEN as u32 {
            assert_eq!(buffer[i as usize], i + 1);
        }
        assert_eq!(postings.fill_buffer(&mut buffer), COLLECT_BLOCK_BUFFER_LEN);
        for i in 0u32..COLLECT_BLOCK_BUFFER_LEN as u32 {
            assert_eq!(buffer[i as usize], i + 1 + COLLECT_BLOCK_BUFFER_LEN as u32);
        }
        assert_eq!(postings.fill_buffer(&mut buffer), 9);
    }
}

use crate::error::Result;
use crate::index::{DocId, Index};

pub fn remove_document(index: &mut Index, id: DocId, skip_deletion: bool) -> Result<()> {
    if index.fastupdate {
        if let Some(refs) = get_fastupdate_refs(index, id) {
            remove_fastupdate(index, refs, id)?;
        }
    } else {
        remove_from_index(index, id)?;
    }

    if !skip_deletion {
        match &mut index.reg {
            crate::index::Register::Set(reg) => {
                reg.delete(&id);
            }
            crate::index::Register::Map(reg) => {
                reg.delete(&id);
            }
        }
    }

    index.documents.remove(&id);

    Ok(())
}

fn get_fastupdate_refs(index: &Index, id: DocId) -> Option<Vec<crate::index::IndexRef>> {
    match &index.reg {
        crate::index::Register::Map(reg) => {
            let id_hash = index.keystore_hash_str(&id.to_string());
            if let Some(id_map) = reg.index.get(&id_hash) {
                if let Some(refs) = id_map.get(&id) {
                    return Some(refs.clone());
                }
            }
            None
        }
        _ => None,
    }
}

fn remove_fastupdate(
    index: &mut Index,
    refs: Vec<crate::index::IndexRef>,
    id: DocId,
) -> Result<()> {
    // Based on the implementation in the JavaScript version, we directly manipulate the indexed array stored in the reg
    // Use the key information in IndexRef to locate to the corresponding index array
    for index_ref in refs {
        match index_ref {
            crate::index::IndexRef::MapRef(term) => {
                let term_hash = index.keystore_hash_str(&term);
                if let Some(term_map) = index.map.index.get_mut(&term_hash) {
                    if let Some(doc_ids) = term_map.get_mut(&term) {
                        // Check whether the last element of the array is the target ID, and if it is, remove it directly (optimization for the JavaScript version)
                        if doc_ids.last() == Some(&id) {
                            doc_ids.pop();
                        } else {
                            // Otherwise find and remove the specified ID
                            if let Some(pos) = doc_ids.iter().position(|x| x == &id) {
                                if doc_ids.len() > 1 {
                                    doc_ids.swap_remove(pos);
                                } else {
                                    doc_ids.clear();
                                }
                            }
                        }
                    }
                }
            }
            crate::index::IndexRef::CtxRef(keyword, term) => {
                let kw_hash = index.keystore_hash_str(&keyword);
                if let Some(term_map) = index.ctx.index.get_mut(&kw_hash) {
                    if let Some(doc_ids) = term_map.get_mut(&term) {
                        // Check whether the last element of the array is the target ID, and if it is, remove it directly (optimization for the JavaScript version)
                        if doc_ids.last() == Some(&id) {
                            doc_ids.pop();
                        } else {
                            // Otherwise find and remove the specified ID
                            if let Some(pos) = doc_ids.iter().position(|x| x == &id) {
                                if doc_ids.len() > 1 {
                                    doc_ids.swap_remove(pos);
                                } else {
                                    doc_ids.clear();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn remove_from_index(index: &mut Index, id: DocId) -> Result<()> {
    let mut terms_to_remove_map = Vec::new();
    let mut terms_to_remove_ctx = Vec::new();

    for (&term_hash, term_map) in index.map.index.iter_mut() {
        for (term, doc_ids) in term_map.iter_mut() {
            if let Some(pos) = doc_ids.iter().position(|x| *x == id) {
                if doc_ids.len() > 1 {
                    doc_ids.swap_remove(pos);
                } else {
                    terms_to_remove_map.push((term_hash, term.clone()));
                }
            }
        }
    }

    for (term_hash, term) in terms_to_remove_map {
        if let Some(map) = index.map.index.get_mut(&term_hash) {
            map.remove(&term);
        }
    }

    for (&term_hash, term_map) in index.ctx.index.iter_mut() {
        for (term, doc_ids) in term_map.iter_mut() {
            if let Some(pos) = doc_ids.iter().position(|x| *x == id) {
                if doc_ids.len() > 1 {
                    doc_ids.swap_remove(pos);
                } else {
                    terms_to_remove_ctx.push((term_hash, term.clone()));
                }
            }
        }
    }

    for (term_hash, term) in terms_to_remove_ctx {
        if let Some(map) = index.ctx.index.get_mut(&term_hash) {
            map.remove(&term);
        }
    }

    Ok(())
}

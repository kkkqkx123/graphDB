//! KeyUtils - 键编码工具

use super::error::{CodecError, Result};

pub struct KeyUtils;

impl KeyUtils {
    const KEY_TYPE_TAG: u32 = 0x00000001;
    const KEY_TYPE_EDGE: u32 = 0x00000002;
    const KEY_TYPE_VERTEX: u32 = 0x00000003;
    const KEY_TYPE_SYSTEM: u32 = 0x00000004;
    const KEY_TYPE_INDEX: u32 = 0x00000005;

    pub fn encode_vertex_key(
        vid_len: usize,
        _space_id: u32,
        part_id: u32,
        vid: &[u8],
    ) -> Vec<u8> {
        let mut key = Vec::with_capacity(4 + vid_len);

        let item = (part_id << 8) | Self::KEY_TYPE_VERTEX;
        key.extend_from_slice(&item.to_le_bytes());

        key.extend_from_slice(vid);
        if vid.len() < vid_len {
            key.extend(vec![0u8; vid_len - vid.len()]);
        }

        key
    }

    pub fn encode_tag_key(
        vid_len: usize,
        _space_id: u32,
        part_id: u32,
        vid: &[u8],
        tag_id: u32,
    ) -> Vec<u8> {
        let mut key = Vec::with_capacity(4 + 4 + vid_len + 4);

        let item = (part_id << 8) | Self::KEY_TYPE_TAG;
        key.extend_from_slice(&item.to_le_bytes());

        key.extend_from_slice(vid);
        if vid.len() < vid_len {
            key.extend(vec![0u8; vid_len - vid.len()]);
        }

        key.extend_from_slice(&tag_id.to_le_bytes());

        key
    }

    pub fn encode_edge_key(
        vid_len: usize,
        _space_id: u32,
        part_id: u32,
        src_vid: &[u8],
        edge_type: u32,
        rank: i64,
        dst_vid: &[u8],
    ) -> Vec<u8> {
        let mut key = Vec::with_capacity(4 + 4 + vid_len + 4 + 8 + vid_len + 1);

        let item = (part_id << 8) | Self::KEY_TYPE_EDGE;
        key.extend_from_slice(&item.to_le_bytes());

        key.extend_from_slice(src_vid);
        if src_vid.len() < vid_len {
            key.extend(vec![0u8; vid_len - src_vid.len()]);
        }

        key.extend_from_slice(&edge_type.to_le_bytes());

        key.extend_from_slice(&rank.to_le_bytes());

        key.extend_from_slice(dst_vid);
        if dst_vid.len() < vid_len {
            key.extend(vec![0u8; vid_len - dst_vid.len()]);
        }

        key.push(0x01);

        key
    }

    pub fn decode_vertex_key(key: &[u8], vid_len: usize) -> Result<(u32, Vec<u8>)> {
        if key.len() < 4 + vid_len {
            return Err(CodecError::InvalidData("Key too short".to_string()));
        }

        let item = u32::from_le_bytes(key[0..4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to decode vertex key".to_string())
        })?);
        let _key_type = item & 0xFF;

        let vid = key[4..4 + vid_len].to_vec();

        Ok((item >> 8, vid))
    }

    pub fn decode_edge_key(key: &[u8], vid_len: usize) -> Result<(u32, Vec<u8>, i32, i64, Vec<u8>)> {
        let min_len = 4 + 4 + vid_len + 4 + 8 + vid_len + 1;
        if key.len() < min_len {
            return Err(CodecError::InvalidData("Key too short".to_string()));
        }

        let item = u32::from_le_bytes(key[0..4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to decode edge key".to_string())
        })?);

        let src_vid_offset = 4;
        let src_vid = key[src_vid_offset..src_vid_offset + vid_len].to_vec();

        let edge_type_offset = 4 + vid_len;
        let edge_type = i32::from_le_bytes(key[edge_type_offset..edge_type_offset + 4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to decode edge type".to_string())
        })?);

        let rank_offset = edge_type_offset + 4;
        let rank = i64::from_le_bytes(key[rank_offset..rank_offset + 8].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to decode rank".to_string())
        })?);

        let dst_vid_offset = rank_offset + 8;
        let dst_vid = key[dst_vid_offset..dst_vid_offset + vid_len].to_vec();

        Ok((item >> 8, src_vid, edge_type, rank, dst_vid))
    }

    pub fn vertex_prefix(vid_len: usize, part_id: u32) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(4);
        let item = (part_id << 8) | Self::KEY_TYPE_VERTEX;
        prefix.extend_from_slice(&item.to_le_bytes());
        prefix
    }

    pub fn tag_prefix(vid_len: usize, part_id: u32, vid: &[u8]) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(4 + vid_len);
        let item = (part_id << 8) | Self::KEY_TYPE_TAG;
        prefix.extend_from_slice(&item.to_le_bytes());
        prefix.extend_from_slice(vid);
        if vid.len() < vid_len {
            prefix.extend(vec![0u8; vid_len - vid.len()]);
        }
        prefix
    }

    pub fn edge_prefix(vid_len: usize, part_id: u32, src_vid: &[u8]) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(4 + vid_len);
        let item = (part_id << 8) | Self::KEY_TYPE_EDGE;
        prefix.extend_from_slice(&item.to_le_bytes());
        prefix.extend_from_slice(src_vid);
        if src_vid.len() < vid_len {
            prefix.extend(vec![0u8; vid_len - src_vid.len()]);
        }
        prefix
    }

    pub fn is_vertex_key(key: &[u8]) -> bool {
        if key.len() < 4 {
            return false;
        }
        let item = u32::from_le_bytes(key[0..4].try_into().unwrap());
        (item & 0xFF) == Self::KEY_TYPE_VERTEX
    }

    pub fn is_edge_key(key: &[u8]) -> bool {
        if key.len() < 4 {
            return false;
        }
        let item = u32::from_le_bytes(key[0..4].try_into().unwrap());
        (item & 0xFF) == Self::KEY_TYPE_EDGE
    }

    pub fn is_tag_key(key: &[u8]) -> bool {
        if key.len() < 4 {
            return false;
        }
        let item = u32::from_le_bytes(key[0..4].try_into().unwrap());
        (item & 0xFF) == Self::KEY_TYPE_TAG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_vertex_key() {
        let vid = b"player100";
        let key = KeyUtils::encode_vertex_key(16, 1, 1, vid);

        assert_eq!(key.len(), 4 + 16);
        assert_eq!(key[0], 0x03);
        assert_eq!(key[1], 0x01);
        assert_eq!(key[2], 0x00);
        assert_eq!(key[3], 0x00);

        let (part_id, decoded_vid) = KeyUtils::decode_vertex_key(&key, 16).unwrap();
        assert_eq!(part_id, 1);
        assert_eq!(&decoded_vid[..vid.len()], vid);
    }

    #[test]
    fn test_encode_edge_key() {
        let src_vid = b"player100";
        let dst_vid = b"team200";
        let key = KeyUtils::encode_edge_key(16, 1, 1, src_vid, 1, 0, dst_vid);

        assert_eq!(key.len(), 4 + 16 + 4 + 8 + 16 + 1);
        assert_eq!(key[0], 0x02);
    }

    #[test]
    fn test_key_type_detection() {
        let vertex_key = KeyUtils::encode_vertex_key(8, 1, 1, b"player1");
        assert!(KeyUtils::is_vertex_key(&vertex_key));
        assert!(!KeyUtils::is_edge_key(&vertex_key));

        let edge_key = KeyUtils::encode_edge_key(8, 1, 1, b"player1", 1, 0, b"team1");
        assert!(!KeyUtils::is_vertex_key(&edge_key));
        assert!(KeyUtils::is_edge_key(&edge_key));
    }
}

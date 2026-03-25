//! Character set information type definition

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

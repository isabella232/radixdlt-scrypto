use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a lazy map id.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mid(pub H256, pub u32);

/// Represents an error when parsing Mid.
#[derive(Debug, Clone)]
pub enum ParseMidError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl fmt::Display for ParseMidError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseMidError {}

impl Mid {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(36);
        vec.extend(self.0.as_ref());
        vec.extend(&self.1.to_le_bytes());
        vec
    }
}

impl TryFrom<&[u8]> for Mid {
    type Error = ParseMidError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 36 {
            Err(ParseMidError::InvalidLength(slice.len()))
        } else {
            Ok(Self(
                H256(copy_u8_array(&slice[..32])),
                u32::from_le_bytes(copy_u8_array(&slice[32..])),
            ))
        }
    }
}

impl FromStr for Mid {
    type Err = ParseMidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseMidError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Debug for Mid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for Mid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl TypeId for Mid {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_MID
    }
}

impl Encode for Mid {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Mid {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_MID))
    }
}

impl Describe for Mid {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_MID.to_owned(),
            generics: vec![],
        }
    }
}

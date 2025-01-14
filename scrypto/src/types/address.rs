use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;

/// The package which defines the `System` blueprint.
pub const SYSTEM_PACKAGE: Address = Address::Package([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
]);

/// The system component
pub const SYSTEM_COMPONENT: Address = Address::Component([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
]);

/// The package that defines the `Account` blueprint.
pub const ACCOUNT_PACKAGE: Address = Address::Package([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
]);

/// The XRD resource definition.
pub const RADIX_TOKEN: Address = Address::ResourceDef([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
]);

pub const ECDSA_TOKEN: Address = Address::ResourceDef([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
]);

/// Represents an address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Address {
    /// Represents a package.
    Package([u8; 26]),

    /// Represents a component.
    Component([u8; 26]),

    /// Represents a resource definition.
    ResourceDef([u8; 26]),
}

/// Represents an error when parsing Address.
#[derive(Debug, Clone)]
pub enum ParseAddressError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
    InvalidType(u8),
}

impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseAddressError {}

impl Address {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Package(d) => combine(1, d),
            Self::Component(d) => combine(2, d),
            Self::ResourceDef(d) => combine(3, d),
        }
    }

    pub fn is_package(&self) -> bool {
        matches!(self, Address::Package(_))
    }

    pub fn is_component(&self) -> bool {
        matches!(self, Address::Component(_))
    }

    pub fn is_resource_def(&self) -> bool {
        matches!(self, Address::ResourceDef(_))
    }
}

impl FromStr for Address {
    type Err = ParseAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseAddressError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ParseAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match slice[0] {
                1 => Ok(Self::Package(copy_u8_array(&slice[1..]))),
                2 => Ok(Self::Component(copy_u8_array(&slice[1..]))),
                3 => Ok(Self::ResourceDef(copy_u8_array(&slice[1..]))),
                _ => Err(ParseAddressError::InvalidType(slice[0])),
            },
            _ => Err(ParseAddressError::InvalidLength(slice.len())),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl TypeId for Address {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_ADDRESS
    }
}

impl Encode for Address {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Address {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_ADDRESS))
    }
}

impl Describe for Address {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_ADDRESS.to_owned(),
            generics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "037ac8066e51cd0d6b320c338d5abbcdbcca25572b6b3e11ee944a";
        let a = Address::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}

use sbor::*;

use crate::rust::vec::Vec;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    sbor::encode_with_type(Vec::with_capacity(512), v)
}

/// Encodes a data structure into byte array for radix engine.
pub fn scrypto_encode_for_radix_engine<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    // create a buffer and pre-append with length (0).
    let mut buf = Vec::with_capacity(512);
    buf.extend(&[0u8; 4]);

    // encode the data structure
    buf = sbor::encode_with_type(buf, v);

    // update the length field
    let len = (buf.len() - 4) as u32;
    (&mut buf[0..4]).copy_from_slice(&len.to_le_bytes());

    buf
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    sbor::decode_with_type(buf)
}

#[cfg(test)]
mod tests {
    use sbor::*;

    use crate::buffer::*;
    use crate::engine::*;
    use crate::resource::*;
    use crate::rust::borrow::ToOwned;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::types::*;

    #[test]
    fn test_serialization() {
        let obj = PutComponentStateInput {
            state: scrypto_encode(&"test".to_owned()),
        };
        let encoded = crate::buffer::scrypto_encode(&obj);
        let decoded = crate::buffer::scrypto_decode::<PutComponentStateInput>(&encoded).unwrap();
        assert_eq!(scrypto_decode::<String>(&decoded.state).unwrap(), "test");
    }

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = crate::buffer::scrypto_encode_for_radix_engine("abc");
        assert_eq!(vec![8, 0, 0, 0, 12, 3, 0, 0, 0, 97, 98, 99], encoded);
    }

    #[derive(TypeId, Encode, Decode)]
    struct ComponentTest {
        resource_address: Address,
        bucket: Bucket,
        secret: String,
    }
}

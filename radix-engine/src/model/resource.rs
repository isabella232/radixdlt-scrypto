use sbor::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::types::*;

/// Represents a resource created.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Resource {
    pub metadata: HashMap<String, String>,
    pub minter: Option<Address>,
    pub supply: Option<Amount>,
}

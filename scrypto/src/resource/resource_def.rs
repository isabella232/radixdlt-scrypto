use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::engine::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// Represents the definition of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDef {
    address: Address,
}

impl From<Address> for ResourceDef {
    fn from(address: Address) -> Self {
        if !address.is_resource_def() {
            panic!("{} is not a resource definition address", address);
        }

        Self { address }
    }
}

impl From<ResourceDef> for Address {
    fn from(a: ResourceDef) -> Address {
        a.address
    }
}

impl ResourceDef {
    /// Creates a resource with the given parameters.
    ///
    /// A bucket is returned iif an initial supply is provided.
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        flags: u64,
        mutable_flags: u64,
        authorities: HashMap<Address, u64>,
        initial_supply: Option<NewSupply>,
    ) -> (ResourceDef, Option<Bucket>) {
        let input = CreateResourceInput {
            resource_type,
            metadata,
            flags,
            mutable_flags,
            authorities,
            initial_supply,
        };
        let output: CreateResourceOutput = call_engine(CREATE_RESOURCE, input);

        (
            output.resource_address.into(),
            output.bucket.map(Into::into),
        )
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T, auth: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_address: self.address,
            new_supply: NewSupply::Fungible {
                amount: amount.into(),
            },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        output.bid.into()
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(
        &mut self,
        key: &NonFungibleKey,
        data: T,
        auth: BucketRef,
    ) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(key.clone(), (data.immutable_data(), data.mutable_data()));

        let input = MintResourceInput {
            resource_address: self.address,
            new_supply: NewSupply::NonFungible { entries },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_engine(MINT_RESOURCE, input);

        output.bid.into()
    }

    /// Burns a bucket of resources.
    pub fn burn(&mut self, bucket: Bucket) {
        let input = BurnResourceInput {
            bid: bucket.into(),
            auth: None,
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Burns a bucket of resources.
    pub fn burn_with_auth(&mut self, bucket: Bucket, auth: BucketRef) {
        let input = BurnResourceInput {
            bid: bucket.into(),
            auth: Some(auth.into()),
        };
        let _output: BurnResourceOutput = call_engine(BURN_RESOURCE, input);
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_address: self.address,
        };
        let output: GetResourceTypeOutput = call_engine(GET_RESOURCE_TYPE, input);

        output.resource_type
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_address: self.address,
        };
        let output: GetResourceMetadataOutput = call_engine(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the feature flags.
    pub fn flags(&self) -> u64 {
        let input = GetResourceFlagsInput {
            resource_address: self.address,
        };
        let output: GetResourceFlagsOutput = call_engine(GET_RESOURCE_FLAGS, input);

        output.flags
    }

    /// Returns the mutable feature flags.
    pub fn mutable_flags(&self) -> u64 {
        let input = GetResourceMutableFlagsInput {
            resource_address: self.address,
        };
        let output: GetResourceMutableFlagsOutput = call_engine(GET_RESOURCE_MUTABLE_FLAGS, input);

        output.mutable_flags
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = GetResourceTotalSupplyInput {
            resource_address: self.address,
        };
        let output: GetResourceTotalSupplyOutput = call_engine(GET_RESOURCE_TOTAL_SUPPLY, input);

        output.total_supply
    }

    /// Returns the address of this resource.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, key: &NonFungibleKey) -> T {
        let input = GetNonFungibleDataInput {
            resource_address: self.address,
            key: key.clone(),
        };
        let output: GetNonFungibleDataOutput = call_engine(GET_NON_FUNGIBLE_DATA, input);

        scrypto_unwrap(T::decode(&output.immutable_data, &output.mutable_data))
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        key: &NonFungibleKey,
        new_data: T,
        auth: BucketRef,
    ) {
        let input = UpdateNonFungibleMutableDataInput {
            resource_address: self.address,
            key: key.clone(),
            new_mutable_data: new_data.mutable_data(),
            auth: auth.into(),
        };
        let _: UpdateNonFungibleMutableDataOutput =
            call_engine(UPDATE_NON_FUNGIBLE_MUTABLE_DATA, input);
    }

    /// Turns on feature flags.
    pub fn enable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_address: self.address,
            new_flags: self.flags() | flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Turns off feature flags.
    pub fn disable_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceFlagsInput {
            resource_address: self.address,
            new_flags: self.flags() & !flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceFlagsOutput = call_engine(UPDATE_RESOURCE_FLAGS, input);
    }

    /// Locks feature flag settings.
    pub fn lock_flags(&mut self, flags: u64, auth: BucketRef) {
        let input = UpdateResourceMutableFlagsInput {
            resource_address: self.address,
            new_mutable_flags: self.flags() & !flags,
            auth: auth.into(),
        };
        let _output: UpdateResourceMutableFlagsOutput =
            call_engine(UPDATE_RESOURCE_MUTABLE_FLAGS, input);
    }

    pub fn update_metadata(&mut self, new_metadata: HashMap<String, String>, auth: BucketRef) {
        let input = UpdateResourceMetadataInput {
            resource_address: self.address,
            new_metadata,
            auth: auth.into(),
        };
        let _output: UpdateResourceMetadataOutput = call_engine(UPDATE_RESOURCE_METADATA, input);
    }
}

//========
// SBOR
//========

impl TypeId for ResourceDef {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for ResourceDef {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for ResourceDef {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for ResourceDef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_RESOURCE_DEF.to_owned(),
            generics: vec![],
        }
    }
}

use lru::LruCache;
use scrypto::engine::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track is shared by all the processes created within a transaction.
///
pub struct Track<'s, S: SubstateStore> {
    ledger: &'s mut S,
    transaction_hash: H256,
    transaction_signers: Vec<EcdsaPublicKey>,
    id_allocator: IdAllocator,
    logs: Vec<(LogLevel, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    resource_defs: HashMap<Address, ResourceDef>,
    lazy_maps: HashMap<(Address, Mid), LazyMap>,
    vaults: HashMap<(Address, Vid), Vault>,
    non_fungibles: HashMap<(Address, NonFungibleKey), NonFungible>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_lazy_maps: HashSet<(Address, Mid)>,
    updated_resource_defs: HashSet<Address>,
    updated_vaults: HashSet<(Address, Vid)>,
    updated_non_fungibles: HashSet<(Address, NonFungibleKey)>,
    new_entities: Vec<Address>,
    code_cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'s, S: SubstateStore> Track<'s, S> {
    pub fn new(
        ledger: &'s mut S,
        transaction_hash: H256,
        transaction_signers: Vec<EcdsaPublicKey>,
    ) -> Self {
        Self {
            ledger,
            transaction_hash,
            transaction_signers,
            id_allocator: IdAllocator::new(IdSpace::Application),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            resource_defs: HashMap::new(),
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_lazy_maps: HashSet::new(),
            updated_resource_defs: HashSet::new(),
            updated_vaults: HashSet::new(),
            updated_non_fungibles: HashSet::new(),
            new_entities: Vec::new(),
            code_cache: LruCache::new(1024),
        }
    }

    /// Start a process.
    pub fn start_process<'r>(&'r mut self, verbose: bool) -> Process<'r, 's, S> {
        // FIXME: This is a temp solution
        let signers: BTreeSet<NonFungibleKey> = self
            .transaction_signers
            .clone()
            .into_iter()
            .map(|key| NonFungibleKey::new(key.to_vec()))
            .collect();
        let mut process = Process::new(0, verbose, self);

        // Always create a virtual bucket of signatures even if there is none.
        // This is to make reasoning at transaction manifest & validator easier.
        let ecdsa_bucket = Bucket::new(
            ECDSA_TOKEN,
            ResourceType::NonFungible,
            Supply::NonFungible { keys: signers },
        );
        process.create_virtual_bucket_ref(ECDSA_TOKEN_BID, ECDSA_TOKEN_RID, ecdsa_bucket);

        process
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> H256 {
        self.transaction_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.ledger.get_epoch()
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(LogLevel, String)> {
        &self.logs
    }

    /// Returns new entities created so far.
    pub fn new_entities(&self) -> &[Address] {
        &self.new_entities
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(address).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&address) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(address, module);
                    Some(inst)
                }
            }
            None => None,
        }
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, address: Address) -> Option<&Package> {
        if self.packages.contains_key(&address) {
            return self.packages.get(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, address: Address) -> Option<&mut Package> {
        self.updated_packages.insert(address);

        if self.packages.contains_key(&address) {
            return self.packages.get_mut(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, address: Address, package: Package) {
        self.updated_packages.insert(address);

        self.packages.insert(address, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, address: Address) -> Option<&Component> {
        if self.components.contains_key(&address) {
            return self.components.get(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get(&address)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, address: Address) -> Option<&mut Component> {
        self.updated_components.insert(address);

        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, address: Address, component: Component) {
        self.updated_components.insert(address);

        self.components.insert(address, component);
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<&NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_address, key.clone()))
        {
            return self.non_fungibles.get(&(resource_address, key.clone()));
        }

        if let Some(non_fungible) = self.ledger.get_non_fungible(resource_address, key) {
            self.non_fungibles
                .insert((resource_address, key.clone()), non_fungible);
            self.non_fungibles.get(&(resource_address, key.clone()))
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<&mut NonFungible> {
        self.updated_non_fungibles
            .insert((resource_address, key.clone()));

        if self
            .non_fungibles
            .contains_key(&(resource_address, key.clone()))
        {
            return self.non_fungibles.get_mut(&(resource_address, key.clone()));
        }

        if let Some(non_fungible) = self.ledger.get_non_fungible(resource_address, key) {
            self.non_fungibles
                .insert((resource_address, key.clone()), non_fungible);
            self.non_fungibles.get_mut(&(resource_address, key.clone()))
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.updated_non_fungibles
            .insert((resource_address, key.clone()));

        self.non_fungibles
            .insert((resource_address, key.clone()), non_fungible);
    }

    /// Returns an immutable reference to a lazy map, if exists.
    pub fn get_lazy_map(&mut self, component_address: &Address, mid: &Mid) -> Option<&LazyMap> {
        let lazy_map_id = (component_address.clone(), mid.clone());

        if self.lazy_maps.contains_key(&lazy_map_id) {
            return self.lazy_maps.get(&lazy_map_id);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(component_address, mid) {
            self.lazy_maps.insert(lazy_map_id, lazy_map);
            self.lazy_maps.get(&lazy_map_id)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a lazy map, if exists.
    pub fn get_lazy_map_mut(
        &mut self,
        component_address: &Address,
        mid: &Mid,
    ) -> Option<&mut LazyMap> {
        let lazy_map_id = (component_address.clone(), mid.clone());
        self.updated_lazy_maps.insert(lazy_map_id.clone());

        if self.lazy_maps.contains_key(&lazy_map_id) {
            return self.lazy_maps.get_mut(&lazy_map_id);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(component_address, mid) {
            self.lazy_maps.insert(lazy_map_id, lazy_map);
            self.lazy_maps.get_mut(&lazy_map_id)
        } else {
            None
        }
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map(&mut self, component_address: Address, mid: Mid, lazy_map: LazyMap) {
        let lazy_map_id = (component_address, mid);
        self.updated_lazy_maps.insert(lazy_map_id.clone());
        self.lazy_maps.insert(lazy_map_id, lazy_map);
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, address: Address) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(&mut self, address: Address) -> Option<&mut ResourceDef> {
        self.updated_resource_defs.insert(address);

        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get_mut(&address);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.updated_resource_defs.insert(address);

        self.resource_defs.insert(address, resource_def);
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, component_address: &Address, vid: &Vid) -> Option<&mut Vault> {
        let vault_id = (component_address.clone(), vid.clone());
        self.updated_vaults.insert(vault_id.clone());

        if self.vaults.contains_key(&vault_id) {
            return self.vaults.get_mut(&vault_id);
        }

        if let Some(vault) = self.ledger.get_vault(component_address, vid) {
            self.vaults.insert(vault_id, vault);
            self.vaults.get_mut(&vault_id)
        } else {
            None
        }
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, component_address: Address, vid: Vid, vault: Vault) {
        let vault_id = (component_address, vid);
        self.updated_vaults.insert(vault_id);
        self.vaults.insert(vault_id, vault);
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self) -> Address {
        // Security Alert: ensure ID allocating will practically never fail
        let address = self
            .id_allocator
            .new_package_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_component_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new resource definition address.
    pub fn new_resource_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_resource_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash()).unwrap()
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Bid {
        self.id_allocator.new_bid().unwrap()
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self) -> Vid {
        self.id_allocator.new_vid(self.transaction_hash()).unwrap()
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> Rid {
        self.id_allocator.new_rid().unwrap()
    }

    /// Creates a new map id.
    pub fn new_mid(&mut self) -> Mid {
        self.id_allocator.new_mid(self.transaction_hash()).unwrap()
    }

    /// Commits changes to the underlying ledger.
    pub fn commit(&mut self) {
        for address in self.updated_packages.clone() {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        for address in self.updated_components.clone() {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        for address in self.updated_resource_defs.clone() {
            self.ledger
                .put_resource_def(address, self.resource_defs.get(&address).unwrap().clone());
        }

        for (component_address, mid) in self.updated_lazy_maps.clone() {
            let lazy_map = self
                .lazy_maps
                .get(&(component_address, mid))
                .unwrap()
                .clone();
            self.ledger.put_lazy_map(component_address, mid, lazy_map);
        }

        for (component_address, vid) in self.updated_vaults.clone() {
            let vault = self.vaults.get(&(component_address, vid)).unwrap().clone();
            self.ledger.put_vault(component_address, vid, vault);
        }

        for (resource_def, id) in self.updated_non_fungibles.clone() {
            self.ledger.put_non_fungible(
                resource_def,
                &id,
                self.non_fungibles
                    .get(&(resource_def, id.clone()))
                    .unwrap()
                    .clone(),
            );
        }
    }
}

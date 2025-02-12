use scrypto::prelude::*;
use scrypto::engine::*;

blueprint! {
    struct CyclicMap {
        maps: LazyMap<u32, LazyMap<u32, u32>>
    }

    impl CyclicMap {
        pub fn new() -> Component {
            let map0 = LazyMap::new();
            let map1 = LazyMap::new();
            map0.insert(1u32, map1);

            let input = PutLazyMapEntryInput {
                mid: Mid(Context::transaction_hash(), 1025),
                key: scrypto_encode(&0u32),
                value: scrypto_encode(&Mid(Context::transaction_hash(), 1024)),
            };
            let _: PutLazyMapEntryOutput = call_engine(PUT_LAZY_MAP_ENTRY, input);

            CyclicMap {
                maps: map0
            }.instantiate()
        }

        pub fn new_self_cyclic() -> Component {
            let map0 = LazyMap::new();

            let input = PutLazyMapEntryInput {
                mid: Mid(Context::transaction_hash(), 1024),
                key: scrypto_encode(&0u32),
                value: scrypto_encode(&Mid(Context::transaction_hash(), 1024)),
            };
            let _: PutLazyMapEntryOutput = call_engine(PUT_LAZY_MAP_ENTRY, input);

            CyclicMap {
                maps: map0
            }.instantiate()
        }
    }
}

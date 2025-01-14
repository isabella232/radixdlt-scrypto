use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::resim::*;

/// Mint resource
#[derive(Parser, Debug)]
pub struct Mint {
    /// The amount of resource to mint
    amount: Decimal,

    /// The resource address
    resource_address: Address,

    /// The minter badge address
    badge_address: Address,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<EcdsaPublicKey>>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Mint {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let default_signers = get_default_signers()?;
        let signatures = self.signers.clone().unwrap_or(default_signers);
        let transaction = TransactionBuilder::new(&executor)
            .withdraw_from_account(
                &Resource::Fungible {
                    amount: 1.into(),
                    resource_address: self.badge_address,
                },
                default_account,
            )
            .mint(self.amount, self.resource_address, self.badge_address)
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(signatures)
            .map_err(Error::TransactionConstructionError)?;
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}

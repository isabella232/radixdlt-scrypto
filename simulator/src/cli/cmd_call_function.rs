use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::ledger::*;
use crate::txn::*;
use crate::utils::*;

const ARG_TRACE: &'static str = "TRACE";
const ARG_PACKAGE: &'static str = "PACKAGE";
const ARG_BLUEPRINT: &'static str = "BLUEPRINT";
const ARG_FUNCTION: &'static str = "FUNCTION";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `call-function` subcommand.
pub fn make_call_function_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_FUNCTION)
        .about("Calls a blueprint function")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_PACKAGE)
                .help("Specify the package address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_BLUEPRINT)
                .help("Specify the blueprint name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_FUNCTION)
                .help("Specify the function name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify the arguments, e.g. `123`, `hello` or `1000:01`.")
                .multiple(true),
        )
}

/// Handles a `call-function` request.
pub fn handle_call_function<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let package: Address = matches
        .value_of(ARG_PACKAGE)
        .ok_or(Error::MissingArgument(ARG_PACKAGE.to_owned()))?
        .parse()
        .map_err(|e| Error::InvalidAddress(e))?;
    let blueprint = matches
        .value_of(ARG_BLUEPRINT)
        .ok_or(Error::MissingArgument(ARG_BLUEPRINT.to_owned()))?;
    let function = matches
        .value_of(ARG_FUNCTION)
        .ok_or(Error::MissingArgument(ARG_FUNCTION.to_owned()))?;
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(|e| Error::InvalidAddress(e))?;
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            match build_call_function(
                &mut ledger,
                account,
                package,
                blueprint,
                function,
                &args,
                trace,
            ) {
                Ok(txn) => {
                    let receipt = execute(&mut ledger, txn, trace);
                    dump_receipt(receipt);
                    Ok(())
                }
                Err(e) => Err(Error::ConstructionErr(e)),
            }
        }
        None => Err(Error::NoDefaultAccount),
    }
}
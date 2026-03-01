#![cfg(feature = "execution-server")]

use std::env;
use std::path::PathBuf;

use oris_runtime::kernel::{
    canonical_runtime_api_contract_path, write_runtime_api_contract, RUNTIME_API_CONTRACT_DOC_PATH,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(canonical_runtime_api_contract_path);
    write_runtime_api_contract(&output_path)?;
    eprintln!(
        "Wrote runtime API contract to {} (canonical: {}).",
        output_path.display(),
        RUNTIME_API_CONTRACT_DOC_PATH,
    );
    Ok(())
}

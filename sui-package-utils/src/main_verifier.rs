use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use clap::Parser;
use move_binary_format::file_format::CompiledModule;
use move_bytecode_verifier::verifier;
use sui_package_utils::bcs_json::BcsJsonSchema;
use sui_package_utils::package_id_io::PackagesDir;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    packages_dir: PathBuf,
}

fn load_package_modules(
    package_dir: &PathBuf,
) -> Result<BTreeMap<String, CompiledModule>, Box<dyn Error>> {
    let mut modules = BTreeMap::new();
    // read from bcs.json file
    let bcs_json = fs::read_to_string(package_dir.join("bcs.json"))?;
    let bcs_json: BcsJsonSchema = serde_json::from_str(&bcs_json)?;
    for (module_name, module_bytes) in bcs_json.get_module_map() {
        let module = CompiledModule::deserialize_with_defaults(
            BASE64_STANDARD.decode(module_bytes)?.as_slice(),
        )?;
        modules.insert(module_name.clone(), module);
    }
    Ok(modules)
}

fn get_interesting_caps(module: &CompiledModule) -> Vec<String> {
    let mut interesting_caps: Vec<String> = Vec::new();
    for struct_def in module.struct_defs() {
        let name_handle = module.datatype_handle_at(struct_def.struct_handle);
        let struct_name = module.identifier_at(name_handle.name);

        if struct_name.to_string().ends_with("Cap") {
            if struct_def.declared_field_count().unwrap() as u16 > 1 {
                let package_id = module.address_identifier_at(module.self_handle().address);
                let module_name = module.identifier_at(module.self_handle().name);
                interesting_caps.push(format!(
                    "0x{}::{}::{} (n_fields: {})",
                    package_id.to_string(),
                    module_name.to_string(),
                    struct_name.to_string(),
                    struct_def.declared_field_count().unwrap() as u16
                ));
            }
        }
    }
    interesting_caps
}
fn main() -> Result<(), Box<dyn Error>> {
    let cli_args = Args::parse();
    let packages_dir = PackagesDir::new(cli_args.packages_dir.clone());
    let packages = packages_dir.get_package_directories()?;
    let mut bad_packages: Vec<String> = Vec::new();
    for package in packages {
        let modules = load_package_modules(&package)?;
        for (module_name, module) in modules {
            let interesting_caps = get_interesting_caps(&module);
            for interesting_cap in interesting_caps {
                println!("{}", interesting_cap);
            }
            // TODO can I do anything with vector_Swap?
            let result = verifier::verify_module_unmetered(&module);
            if result.is_err() {
                println!(
                    "Error verifying {} {}",
                    package.to_str().unwrap(),
                    module_name
                );
                bad_packages.push(package.to_str().unwrap().to_string());
            }
        }
    }
    println!("Bad packages: {:?}", bad_packages);
    Ok(())
}

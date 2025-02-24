fn main() {
    let contract_dirs: Vec<&str> = vec!["./contracts"];
    blueprint_sdk::build::utils::soldeer_update();
    blueprint_sdk::build::utils::build_contracts(contract_dirs);
}

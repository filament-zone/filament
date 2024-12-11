use clap::{Arg, Command};
use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Native, CryptoSpec, Spec};

type S = DefaultSpec<MockZkVerifier, MockZkVerifier, Native>;

fn main() {
    let matches = Command::new("filament-hub-cli")
        .arg(Arg::new("address").required(true).index(1))
        .get_matches();
    let address = matches
        .get_one::<String>("address")
        .expect("address is required");
    let hub_address = filament_hub_eth::addr_to_hub_address::<S>(address).unwrap();
    let credential_id = filament_hub_eth::hub_addr_to_credential_id::<
        <<DefaultSpec<MockZkVerifier, MockZkVerifier, Native> as Spec>::CryptoSpec as CryptoSpec>::Hasher,
        S,
    >(&hub_address);

    println!("eth: {}", address);
    println!("hub: {}", hub_address);
    println!("credential id: {}", credential_id);
}

use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{
    clap::{self, Parser as _},
    default_spec::DefaultSpec,
    execution_mode::Native,
    prelude::serde_json,
    schemars::schema_for,
};

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum Component {
    #[default]
    Gensis,

    CallMessage,
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t, value_enum)]
    component: Component,
}

fn main() {
    let args = Args::parse();

    let schema = match args.component {
        Component::Gensis => schema_for!(
            filament_hub_core::CoreConfig<DefaultSpec<MockZkVerifier, MockZkVerifier, Native>>
        ),
        Component::CallMessage => schema_for!(
            filament_hub_core::CallMessage<DefaultSpec<MockZkVerifier, MockZkVerifier, Native>>
        ),
    };

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

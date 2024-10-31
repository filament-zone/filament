use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{
    clap::{self, Parser as _},
    default_spec::DefaultSpec,
    execution_mode::Native,
    prelude::serde_json,
    schemars::schema_for,
};

type S = DefaultSpec<MockZkVerifier, MockZkVerifier, Native>;

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum Component {
    #[default]
    Gensis,

    CallMessage,

    Campaign,
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
        Component::Gensis => schema_for!(filament_hub_core::CoreConfig<S>),
        Component::CallMessage => schema_for!(filament_hub_core::CallMessage<S>),
        Component::Campaign => schema_for!(filament_hub_core::campaign::Campaign<S>),
    };

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

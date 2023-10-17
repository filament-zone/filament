use directories::ProjectDirs;

use crate::client::Client;

pub struct Context {
    pub cometbft_uri: String,

    pub client: Client,
    pub dirs: ProjectDirs,
}

impl Context {
    pub fn new(cometbft_uri: String) -> eyre::Result<Self> {
        let client = Client::new(&cometbft_uri)?;
        let dirs = ProjectDirs::from("zone", "filament", "flt")
            .ok_or(eyre::eyre!("no $HOME directory found"))?;

        Ok(Self {
            cometbft_uri,
            client,
            dirs,
        })
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // FIXME(xla): Config should be properly parsed from cli arguments, similar to how plz works.
    pulzaard::run(Default::default()).await
}

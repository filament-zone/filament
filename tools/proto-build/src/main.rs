use std::{io::Result, path::PathBuf};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());

    let target_dir = root
        .join("..")
        .join("..")
        .join("proto")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    let tmpdir = tempfile::tempdir().unwrap();
    let descriptor_path = tmpdir.path().to_owned().join("proto_descriptor.bin");

    let mut config = prost_build::Config::new();
    config.out_dir(&target_dir);
    config
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types();

    config
        .compile_protos(
            &[
                "../proto/proto/pulzaar/core/chain/v1alpha1/chain.proto",
                "../proto/proto/pulzaar/core/stake/v1alpha1/stake.proto",
            ],
            &["../proto/proto/"],
        )
        .unwrap();

    Ok(())
}

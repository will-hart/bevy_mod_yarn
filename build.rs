use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/minimal.yarn");

    if let Err(e) = Command::new("./ysc")
        .arg("compile")
        .arg("-o")
        .arg("./assets")
        .arg("./assets/minimal.yarn")
        .output()
    {
        eprintln!("Failed to compile, maybe ysc wasn't in the root directory? Error: {e:?}")
    }


    // we need to rename the *-Lines.csv and *-Metadata.csv files
    // as bevy currently doesn't support loading multiple asset types
    // with the same extension.
    std::fs::rename("./assets/minimal-Lines.csv", "./assets/minimal.lines.csv")
        .expect("rename Lines CSV files");
    std::fs::rename(
        "./assets/minimal-Metadata.csv",
        "./assets/minimal.metadata.csv",
    )
}

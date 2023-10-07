use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/minimal.yarn");
    println!("cargo:rerun-if-changed=assets/kitchen_sink.yarn");

    if let Err(e) = Command::new("./ysc")
        .arg("compile")
        .arg("-o")
        .arg("./assets")
        .arg("./assets/minimal.yarn")
        .output()
    {
        eprintln!("Failed to compile, maybe ysc wasn't in the root directory? Error: {e:?}")
    }

    if let Err(e) = Command::new("./ysc")
        .arg("compile")
        .arg("-o")
        .arg("./assets")
        .arg("./assets/kitchen_sink.yarn")
        .output()
    {
        eprintln!("Failed to compile, maybe ysc wasn't in the root directory? Error: {e:?}")
    }

    // we need to rename the *-Lines.csv and *-Metadata.csv files
    // as bevy currently doesn't support loading multiple asset types
    // with the same extension. This is really only important for running the examples,
    // so we're just ignoring errors :shrug:
    let _ = std::fs::rename("./assets/minimal-Lines.csv", "./assets/minimal.lines.csv");
    let _ = std::fs::rename(
        "./assets/minimal-Metadata.csv",
        "./assets/minimal.metadata.csv",
    );
    let _ = std::fs::rename(
        "./assets/kitchen_sink-Lines.csv",
        "./assets/kitchen_sink.lines.csv",
    );
    let _ = std::fs::rename(
        "./assets/kitchen_sink-Metadata.csv",
        "./assets/kitchen_sink.metadata.csv",
    );
}

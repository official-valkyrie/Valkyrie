/*
    This file is part of Valkyrie.

    Copyright (C) 2026 Tomasz Jaworski and Valkyrie Contributors

    Valkyrie is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Valkyrie is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Valkyrie.  If not, see <https://gnu.org>.

    Additional terms under GNU GPL version 3 section 7

    1. Linking Exception:
    If you modify this Program, or any covered work, by linking or
    combining it with the ONNX Runtime and/or NVIDIA TensorRT libraries
    (or a modified version of those libraries), containing parts covered
    by the terms of the respective ONNX Runtime and NVIDIA license
    agreements, the licensors of this Program grant you additional
    permission to convey the resulting work.

   2. Attribution Requirement (Section 7b):
    Under Section 7(b) of the GNU General Public License, version 3,
    the licensors of this Program require that any modified or redistributed
    versions of this software must prominently preserve original author
    attributions and credit to the Valkyrie Contributors in the software
    documentation.
*/

#[cfg(not(feature = "release"))]
fn main() {
    use std::process::Command;

    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|text| text.trim().to_owned())
    };

    let commit = git(&[
        "log",
        "-1",
        "--format=%cd-%h",
        "--date=format:%Y%m%d",
        "--abbrev=8",
    ])
    .unwrap_or_else(|| "nogit".to_owned());

    println!("cargo:rustc-env=ENGINE_NAME=Valkyrie-dev-{commit}");

    println!("cargo:rerun-if-changed=build.rs");

    if let Some(head_log) = git(&["rev-parse", "--git-path", "logs/HEAD"]) {
        println!("cargo:rerun-if-changed={head_log}");
    }
}

#[cfg(feature = "release")]
fn main() {
    let formatted_name = std::env::var("ENGINE_NAME")
        .unwrap_or_else(|_| format!("Valkyrie v{}", env!("CARGO_PKG_VERSION")));

    println!("cargo:rustc-env=ENGINE_NAME={formatted_name}");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=ENGINE_NAME");
}

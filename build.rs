use std::io;
use std::process::{self, Command, ExitStatus};

trait StatusExt {
    fn exit_on_failure(&self) {}
}

impl StatusExt for io::Result<ExitStatus> {
    fn exit_on_failure(&self) {
        let Ok(status) = self else {
            println!("cargo:warning={:?}", self);
            process::exit(1);
        };

        let Some(code) = status.code() else {
            process::exit(1);
        };

        if !status.success() {
            process::exit(code);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=templates");

    Command::new("tailwindcss")
        .args([
            "-i",
            "templates/style.css",
            "-o",
            "target/coffee-webui/style.css",
        ])
        .status()
        .exit_on_failure();

    Command::new("esbuild")
        .args([
            "templates/main.ts",
            "--outfile=target/coffee-webui/main.js",
            "--bundle",
            "--minify",
        ])
        .status()
        .exit_on_failure();
}

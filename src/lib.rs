//! Verify SVGs against W3C's DTD to ensure they are according to spec.
//!
//! This calls out to the `xmllint` tool from libxml2, which may not be
//! available.
//!
//! The DTD used here has been patched to verify the `railroad`-debug
//! attributes, as I don't care enough to write a proper XML-module...

extern crate tempfile;

use std::process;
use std::io::{self, Write};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Lint(String)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

pub struct Verifier {
    dtd_tempfile: tempfile::NamedTempFile
}

impl Verifier {
    pub fn new() -> io::Result<Self> {
        let mut dtd_tempfile = tempfile::NamedTempFile::new()?;
        dtd_tempfile.write_all(include_bytes!("svg11-flat.dtd"))?;
        Ok(Verifier { dtd_tempfile })
    }

    pub fn verify(&self, svg_src: String) -> Result<(), Error> {
        let mut child = process::Command::new("xmllint")
                        .arg("--noout")
                        .arg("--dtdvalid")
                        .arg(self.dtd_tempfile.path().as_os_str())
                        .arg("-")
                        .stdin(process::Stdio::piped())
                        .stdout(process::Stdio::null())
                        .stderr(process::Stdio::piped())
                        .spawn()
                        .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let writer = ::std::thread::spawn(move || {
            stdin.write_all(svg_src.as_bytes())
        });
        let output = child.wait_with_output()?;
        writer.join().unwrap()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Lint(String::from_utf8_lossy(&output.stderr).to_string()))
        }
    }
}

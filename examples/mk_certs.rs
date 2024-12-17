#![allow(clippy::uninlined_format_args)]

//! A program that generates ca certs, certs verified by the ca, and public
//! and private keys.

use openssl::error::ErrorStack;
use openssl::x509::X509VerifyResult;

fn real_main() -> Result<(), ErrorStack> {
    let (ca_cert, ca_key_pair) = mqtt_bench::cert::mk_ca_cert()?;
    let (cert, _key_pair) = mqtt_bench::cert::mk_ca_signed_cert(&ca_cert, &ca_key_pair, "example.com")?;

    // Verify that this cert was issued by this ca
    match ca_cert.issued(&cert) {
        X509VerifyResult::OK => println!("Certificate verified!"),
        ver_err => println!("Failed to verify certificate: {}", ver_err),
    };

    Ok(())
}

fn main() {
    match real_main() {
        Ok(()) => println!("Finished."),
        Err(e) => println!("Error: {}", e),
    };
}

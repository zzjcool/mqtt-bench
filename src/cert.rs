use anyhow::{Context, Error};
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::Private;
use openssl::pkey::{PKey, PKeyRef};
use openssl::rsa::Rsa;
use openssl::x509::extension::{
    AuthorityKeyIdentifier, BasicConstraints, KeyUsage, SubjectAlternativeName,
    SubjectKeyIdentifier,
};
use openssl::x509::{X509NameBuilder, X509Ref, X509Req, X509ReqBuilder, X509};
use std::path::Path;
use std::{fs, fs::File, io::Read};

fn read_pem(path: &Path) -> Result<Vec<u8>, Error> {
    let mut f = File::open(path).context("Failed to read CA key file")?;
    let metadata = fs::metadata(path).context("Failed to read metadata of CA key file")?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).context("buffer overflow")?;
    Ok(buffer)
}

pub fn load_ca_pkey(key_path: &Path) -> Result<PKey<Private>, Error> {
    let buffer = read_pem(key_path)?;
    let rsa = Rsa::private_key_from_pem(&buffer[..]).context("Failed to read RSA private key")?;
    let pkey = PKey::from_rsa(rsa).context("Failed tp wrap RSA to Private Key")?;
    Ok(pkey)
}

pub fn load_ca_cert(cert_path: &Path) -> Result<X509, Error> {
    let buffer = read_pem(cert_path)?;
    let cert = X509::from_pem(&buffer).context("Failed to read X509 certificate")?;
    Ok(cert)
}

/// Make a CA certificate and private key
pub fn mk_ca_cert() -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "US")?;
    x509_name.append_entry_by_text("ST", "TX")?;
    x509_name.append_entry_by_text("O", "Some CA organization")?;
    x509_name.append_entry_by_text("CN", "ca test")?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}

/// Make a X509 request with the given private key
pub fn mk_request(key_pair: &PKey<Private>, commonName: &str) -> Result<X509Req, ErrorStack> {
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_pubkey(key_pair)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "US")?;
    x509_name.append_entry_by_text("ST", "TX")?;
    x509_name.append_entry_by_text("O", "Some organization")?;
    x509_name.append_entry_by_text("CN", commonName)?;
    let x509_name = x509_name.build();
    req_builder.set_subject_name(&x509_name)?;

    req_builder.sign(key_pair, MessageDigest::sha256())?;
    let req = req_builder.build();
    Ok(req)
}

/// Make a certificate and private key signed by the given CA cert and private key
pub fn mk_ca_signed_cert(
    ca_cert: &X509Ref,
    ca_key_pair: &PKeyRef<Private>,
    common_name: &str,
) -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let req = mk_request(&key_pair, common_name)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(req.subject_name())?;
    cert_builder.set_issuer_name(ca_cert.subject_name())?;
    cert_builder.set_pubkey(&key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().build()?)?;

    cert_builder.append_extension(
        KeyUsage::new()
            .critical()
            .non_repudiation()
            .digital_signature()
            .key_encipherment()
            .build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    let auth_key_identifier = AuthorityKeyIdentifier::new()
        .keyid(false)
        .issuer(false)
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(auth_key_identifier)?;

    let subject_alt_name = SubjectAlternativeName::new()
        .dns("*.example.com")
        .dns("hello.com")
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_alt_name)?;

    cert_builder.sign(ca_key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}

#[cfg(test)]
mod tests {
    use crate::cert::{load_ca_cert, load_ca_pkey, mk_ca_signed_cert};
    use anyhow::Error;
    use std::path::PathBuf;

    #[test]
    fn test_load_ca_pkey() -> Result<(), Error> {
        let mut ca_key_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        ca_key_path_buf.push("assets");
        ca_key_path_buf.push("CA.key");
        let ca_key = load_ca_pkey(&ca_key_path_buf)?;
        assert_eq!(2048, ca_key.bits());
        Ok(())
    }

    #[test]
    fn test_load_ca_cert() -> Result<(), Error> {
        let mut ca_cert_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        ca_cert_path_buf.push("assets");
        ca_cert_path_buf.push("CA.crt");
        let ca_cert = load_ca_cert(&ca_cert_path_buf)?;
        for entry in ca_cert.subject_name().entries() {
            println!("{:?}", entry);
        }
        Ok(())
    }
    
    #[test]
    fn test_mk_ca_signed_cert() -> Result<(), Error> {
        let mut ca_key_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        ca_key_path_buf.push("assets");
        ca_key_path_buf.push("CA.key");
        let ca_key = load_ca_pkey(&ca_key_path_buf)?;
        
        let mut ca_cert_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        ca_cert_path_buf.push("assets");
        ca_cert_path_buf.push("CA.crt");
        let ca_cert = load_ca_cert(&ca_cert_path_buf)?;
        
        let (cert, key) = mk_ca_signed_cert(&ca_cert, &ca_key, "abc.com")?;

        for entry in cert.subject_name().entries() {
            println!("{:?}", entry);
        }
        
        Ok(())
    }
}

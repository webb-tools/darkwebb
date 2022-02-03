use sp_std::vec::Vec;

pub mod ecdsa;

pub trait SigningSystem {
	type Error;

	fn verify(key: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, Self::Error> {
		let public_key = Self::recover_pub_key(msg, sig)?;
		Ok(public_key == *key)
	}

	fn recover_pub_key(msg: &[u8], sig: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

pub struct SignatureVerifier;

impl SigningSystem for SignatureVerifier {
	type Error = ecdsa::EcdsaVerifyError;

	fn recover_pub_key(msg: &[u8], sig: &[u8]) -> Result<Vec<u8>, Self::Error> {
		ecdsa::recover_ecdsa_pub_key(msg, sig)
	}
}

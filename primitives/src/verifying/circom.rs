use super::ethereum_circom::{Proof, VerifyingKey, G1, G2};
use crate::*;
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::Error;
use ark_groth16::{
	verify_proof, PreparedVerifyingKey, Proof as ArkProof, VerifyingKey as ArkVerifyingKey,
};
use ark_std::vec::Vec;
use arkworks_native_gadgets::to_field_elements;
use sp_core::U256;

pub struct CircomVerifierBn254;

#[derive(Debug)]
pub enum CircomError {
	InvalidVerifyingKeyBytes,
	InvalidProofBytes,
	InvalidBuilderConfig,
	ProvingFailure,
	VerifyingFailure,
	ParameterGenerationFailure,
}

impl ark_std::error::Error for CircomError {}

impl core::fmt::Display for CircomError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			CircomError::InvalidVerifyingKeyBytes => write!(f, "Invalid verifying key bytes"),
			CircomError::InvalidProofBytes => write!(f, "Invalid proof bytes"),
			CircomError::InvalidBuilderConfig => write!(f, "Invalid builder config"),
			CircomError::ProvingFailure => write!(f, "Proving failure"),
			CircomError::VerifyingFailure => write!(f, "Verifying failure"),
			CircomError::ParameterGenerationFailure => write!(f, "Parameter generation failure"),
		}
	}
}

macro_rules! read_to_G1 {
	($arr:expr) => {{
		let x = U256::from_big_endian(&$arr[0..32]);
		let y = U256::from_big_endian(&$arr[32..64]);
		G1 { x, y }
	}};
}

macro_rules! read_to_G2 {
	($arr:expr) => {{
		let x0 = U256::from_big_endian(&$arr[0..32]);
		let y0 = U256::from_big_endian(&$arr[32..64]);
		let x1 = U256::from_big_endian(&$arr[64..96]);
		let y1 = U256::from_big_endian(&$arr[96..128]);
		G2 { x: [x0, x1], y: [y0, y1] }
	}};
}

macro_rules! read_to_ic {
	($arr:expr) => {{
		let mut ic = Vec::new();
		let mut temp_arr = $arr.to_vec();
		while !temp_arr.is_empty() {
			let x0 = U256::from_big_endian(&temp_arr[0..32]);
			let y0 = U256::from_big_endian(&temp_arr[32..64]);
			ic.push(G1 { x: x0, y: y0 });
			temp_arr = temp_arr[64..].to_vec();
		}

		ic
	}};
}

// pub struct VerifyingKey {
//     pub alpha1: G1, (x: U256, y: U256)
//     pub beta2: G2,  [(x: U256, y: U256); 2]
//     pub gamma2: G2, [(x: U256, y: U256); 2]
//     pub delta2: G2, [(x: U256, y: U256); 2]
//     pub ic: Vec<G1>, Vec<(x: U256, y: U256)>
// }
pub fn parse_vk_bytes_to_circom_vk(vk_bytes: &[u8]) -> Result<VerifyingKey, Error> {
	if vk_bytes.len() < 448 {
		return Err(CircomError::InvalidVerifyingKeyBytes.into())
	}

	let circom_vk = VerifyingKey {
		alpha1: read_to_G1!(vk_bytes[0..64]),
		beta2: read_to_G2!(vk_bytes[64..192]),
		gamma2: read_to_G2!(vk_bytes[192..320]),
		delta2: read_to_G2!(vk_bytes[320..448]),
		ic: read_to_ic!(vk_bytes[448..]),
	};

	Ok(circom_vk)
}

// pub struct Proof {
//     pub a: G1, (x: U256, y: U256)
//     pub b: G2, [(x: U256, y: U256); 2]
//     pub c: G1, (x: U256, y: U256)
// }
pub fn parse_proof_to_circom_proof(proof_bytes: &[u8]) -> Result<Proof, Error> {
	if proof_bytes.len() != 192 {
		return Err(CircomError::InvalidProofBytes.into())
	}

	let circom_proof = Proof {
		a: read_to_G1!(proof_bytes[0..64]),
		b: read_to_G2!(proof_bytes[64..192]),
		c: read_to_G1!(proof_bytes[192..256]),
	};

	Ok(circom_proof)
}

pub fn verify_groth16(
	vk: &PreparedVerifyingKey<Bn254>,
	public_inputs: &[Fr],
	proof: &ArkProof<Bn254>,
) -> Result<bool, Error> {
	let res = verify_proof(vk, proof, public_inputs)?;
	Ok(res)
}

impl InstanceVerifier for CircomVerifierBn254 {
	fn verify(public_inp_bytes: &[u8], proof_bytes: &[u8], vk_bytes: &[u8]) -> Result<bool, Error> {
		let public_input_field_elts = to_field_elements::<Fr>(public_inp_bytes)?;

		let circom_vk = parse_vk_bytes_to_circom_vk(vk_bytes)?;
		let circom_proof = parse_proof_to_circom_proof(proof_bytes)?;
		let vk = ArkVerifyingKey::<Bn254>::from(circom_vk);
		let proof = ArkProof::<Bn254>::from(circom_proof);
		let res = verify_groth16(&vk.into(), &public_input_field_elts, &proof)?;
		Ok(res)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::verifying::ethereum_circom::*;
	use sp_core::U256;

	#[test]
	fn verifying_key_serialize_deserialize() {
		let vk = VerifyingKey {
			alpha1: G1 { x: U256::from(1), y: U256::from(2) },
			beta2: G2 { x: [U256::from(3), U256::from(4)], y: [U256::from(5), U256::from(6)] },
			gamma2: G2 { x: [U256::from(7), U256::from(8)], y: [U256::from(9), U256::from(10)] },
			delta2: G2 { x: [U256::from(11), U256::from(12)], y: [U256::from(13), U256::from(14)] },
			ic: vec![
				G1 { x: U256::from(15), y: U256::from(16) },
				G1 { x: U256::from(17), y: U256::from(18) },
				G1 { x: U256::from(19), y: U256::from(20) },
				G1 { x: U256::from(21), y: U256::from(22) },
				G1 { x: U256::from(23), y: U256::from(24) },
			],
		};

		let vk_bytes = vk.to_bytes();
		let vk2 = parse_vk_bytes_to_circom_vk(&vk_bytes).unwrap();
		assert_eq!(vk.alpha1, vk2.alpha1);
		assert_eq!(vk.beta2, vk2.beta2);
		assert_eq!(vk.gamma2, vk2.gamma2);
		assert_eq!(vk.delta2, vk2.delta2);
		assert_eq!(vk.ic, vk2.ic);
	}
}

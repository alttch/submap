use sha2::{Digest, Sha256, Sha512};

#[allow(clippy::module_name_repetitions)]
pub type Sha256Digest = [u64; 4];
#[allow(clippy::module_name_repetitions)]
pub type Sha512Digest = [u64; 8];

/// A helper function to quickly calculate sha256 hash as [u64; 4]
///
/// # Panics
///
/// Should not panic
pub fn sha256(data: impl AsRef<[u8]>) -> Sha256Digest {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash_arr: [u8; 32] = hasher.finalize().try_into().unwrap();
    let hash_1 = u64::from_le_bytes(hash_arr[..8].try_into().unwrap());
    let hash_2 = u64::from_le_bytes(hash_arr[8..16].try_into().unwrap());
    let hash_3 = u64::from_le_bytes(hash_arr[16..24].try_into().unwrap());
    let hash_4 = u64::from_le_bytes(hash_arr[24..].try_into().unwrap());
    [hash_1, hash_2, hash_3, hash_4]
}

/// A helper function to quickly calculate sha512 hash as [u64; 8]
///
/// # Panics
///
/// Should not panic
pub fn sha512(data: impl AsRef<[u8]>) -> Sha512Digest {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let hash_arr: [u8; 64] = hasher.finalize().try_into().unwrap();
    let hash_1 = u64::from_le_bytes(hash_arr[..8].try_into().unwrap());
    let hash_2 = u64::from_le_bytes(hash_arr[8..16].try_into().unwrap());
    let hash_3 = u64::from_le_bytes(hash_arr[16..24].try_into().unwrap());
    let hash_4 = u64::from_le_bytes(hash_arr[24..32].try_into().unwrap());
    let hash_5 = u64::from_le_bytes(hash_arr[32..40].try_into().unwrap());
    let hash_6 = u64::from_le_bytes(hash_arr[40..48].try_into().unwrap());
    let hash_7 = u64::from_le_bytes(hash_arr[48..56].try_into().unwrap());
    let hash_8 = u64::from_le_bytes(hash_arr[56..].try_into().unwrap());
    [
        hash_1, hash_2, hash_3, hash_4, hash_5, hash_6, hash_7, hash_8,
    ]
}

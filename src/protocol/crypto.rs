use rand::Rng;

// KLV keys embedded in the Growtopia binary.
const KEY1: &str = "832aac071ffbcfc15bfe1d0a7ad15221";
const KEY2: &str = "709296ddd04fc4074a7b443ecc0799aa";
const KEY3: &str = "623de1e8fff22a2b3e0d7e01593e7c22";
const KEY4: &str = "bb835e5a57e6c88e2449499ca487ced2";
const KEY5: &str = "ea76e4d6009282186063fe9465f2d9ab";

fn md5u(s: &str) -> String {
    format!("{:X}", md5::compute(s.as_bytes()))
}

/// Growtopia's custom rotate-left-5 hash with NUL terminator appended
/// (equivalent to `HashMode::NullTerminated`).
pub fn hash_string(s: &str) -> i32 {
    let mut h: u32 = 0x55555555;
    for b in s.bytes().chain(std::iter::once(0u8)) {
        h = h.rotate_left(5).wrapping_add(b as u32);
    }
    h as i32
}

/// SHA-256 `klv` of the current Windows client (protocol 227).
/// `who` is the RID; an empty RID falls back to the game version.
pub fn compute_klv_sha(game_version: &str, protocol: &str, rid: &str) -> String {
    const SALT1: &str = "73eff5914c61a20a71ada81a6fc7780700fb1c0285659b4899bc172a24c14fc1";
    const SALT2: &str = "3ca373dffbf463bb337e0fd768a2f395b8e417475438916506c721551f32038d";
    const SALT3: &str = "c85df9056ee603b849a93e1ebab5dd5f66e1fb8b2f4a8caef8d13b9f9e013fa4";
    const SALT4: &str = "e9fc40ec08f9ea6393f59c65e37f750aacddf68490c4f92d0d2523a5bc02ea63";
    const MAGIC: u32 = 0x40B3_851F;

    let who = if rid.is_empty() { game_version } else { rid };
    let h1 = sha256_hex(&format!("{}{}", sha256_hex(protocol), SALT1));
    let h2 = sha256_hex(&md5u(&sha256_hex(who)));
    let h3 = sha256_hex(&sha256_hex(&MAGIC.to_string()));
    let h4 = sha256_hex(&md5u(&sha256_hex(protocol)));
    sha256_hex(&format!("{h4}{SALT4}{h3}{SALT3}{h2}{SALT2}{h1}"))
}

fn sha256_hex(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let dig = Sha256::digest(raw.as_bytes());
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

/// Computes the legacy MD5 `klv` used by the web dashboard request.
pub fn compute_klv(game_version: &str, protocol: &str, rid: &str, hash_val: i32) -> String {
    let combined = format!(
        "{}{}{}{}{}{}{}{}{}",
        md5u(&md5u(game_version)),
        KEY1,
        md5u(&md5u(&md5u(protocol))),
        KEY2,
        KEY3,
        md5u(&md5u(rid)),
        KEY4,
        md5u(&md5u(&hash_val.to_string())),
        KEY5,
    );
    md5u(&combined)
}

/// Generate `n` random uppercase hex characters.
pub fn random_hex(n: usize) -> String {
    let mut rng = rand::rng();
    (0..n).map(|_| format!("{:X}", rng.random::<u8>() & 0xF)).collect()
}

/// Generate a random locally-administered unicast MAC address.
pub fn random_mac() -> String {
    let mut rng = rand::rng();
    format!(
        "02:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
    )
}

/// Generates a 32-char uppercase hex RID derived from the current nanosecond timestamp.
pub fn generate_rid() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032X}", nanos)
}

#[cfg(test)]
mod tests {
    use super::compute_klv_sha;

    #[test]
    fn klv_sha_matches_official_client() {
        let klv = compute_klv_sha("5.61", "227", "AABB");
        assert_eq!(
            klv,
            "76e9d5b3e7709c66d52048118554fe5d43270e1185719ef173839e04c82c0e50"
        );
    }
}

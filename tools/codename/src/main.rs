//! Prints a two-word release codename like "Brave Otter".
//!
//! Usage:
//!   codename-gen            # codename seeded by the current time (varies)
//!   codename-gen v1.2.3     # codename deterministically derived from the seed
//!
//! Passing the version as a seed means a given release always maps to the same
//! codename, which is nicer than pure randomness for reproducible releases.

use std::time::{SystemTime, UNIX_EPOCH};

const ADJECTIVES: &[&str] = &[
    "Brave", "Calm", "Clever", "Bold", "Bright", "Swift", "Quiet", "Lucky",
    "Mighty", "Gentle", "Eager", "Noble", "Witty", "Cosmic", "Amber", "Crimson",
    "Golden", "Silent", "Hidden", "Restless", "Wandering", "Radiant", "Frosty",
    "Vivid", "Daring", "Humble", "Lively", "Mellow", "Rugged", "Serene",
];

const ANIMALS: &[&str] = &[
    "Otter", "Falcon", "Badger", "Lynx", "Heron", "Marten", "Beaver", "Raven",
    "Fox", "Wolf", "Owl", "Bison", "Stoat", "Gecko", "Puffin", "Ibis",
    "Tapir", "Quokka", "Narwhal", "Lemur", "Panda", "Mantis", "Salmon",
    "Hawk", "Moose", "Crane", "Viper", "Walrus", "Yak", "Koala",
];

/// FNV-1a hash of the seed string — small, dependency-free, stable.
fn hash(seed: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in seed.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn main() {
    // Seed: the CLI argument if given, otherwise the current unix time.
    let seed = std::env::args().nth(1).unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos().to_string())
            .unwrap_or_else(|_| "0".to_string())
    });

    let h = hash(&seed);
    let adj = ADJECTIVES[(h % ADJECTIVES.len() as u64) as usize];
    // Shift bits so the two indices are independent.
    let animal = ANIMALS[((h >> 32) % ANIMALS.len() as u64) as usize];
    println!("{adj} {animal}");
}

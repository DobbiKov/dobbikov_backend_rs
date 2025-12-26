use dotenvy::dotenv;
use std::env;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

fn get_salt() -> String {
    dotenv().ok();

    env::var("SALT_FOR_HASHING").expect("SALT_FOR_HASHING must be set")
}

pub fn hash_password(password: String) -> String {
    let salt = SaltString::new(&get_salt()).unwrap();

    let argon2 = Argon2::default();

    argon2
        .hash_password(&(password.into_bytes()), &salt)
        .unwrap()
        .hash
        .unwrap()
        .to_string()
}

pub fn verify_password(password: String, hashed: &str) -> bool {
    hash_password(password) == hashed
}

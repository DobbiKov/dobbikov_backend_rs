use super::pass_hashing::hash_password;
mod common;
mod db;

#[test]
fn test_hashing() {
    let test_pass = "hello123".to_string();

    let hash = hash_password(test_pass);
    let correct_hash = "hdTLpbn/NuGo7LDDsMdkZh40wHyEQv9Y1OrSzhfngbQ".to_string();

    assert_eq!(hash, correct_hash);
}

//! Password hashing and verification using Argon2.

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::rand_core::OsRng;

use crate::error::{AuthError, Result};

/// Hashes a password using Argon2id.
///
/// Returns the hashed password as a PHC string that includes the salt.
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::PasswordHashError)?;

    Ok(password_hash.to_string())
}

/// Verifies a password against a stored hash.
///
/// Returns true if the password matches, false otherwise.
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Checks if a password meets minimum security requirements.
pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(AuthError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    // Check for at least one letter and one digit
    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_letter {
        return Err(AuthError::Validation(
            "Password must contain at least one letter".to_string(),
        ));
    }

    if !has_digit {
        return Err(AuthError::Validation(
            "Password must contain at least one digit".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "securepassword123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "securepassword123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Hashes should be different due to different salts
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1));
        assert!(verify_password(password, &hash2));
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("Pass1234").is_ok());

        // Too short
        assert!(validate_password("pass1").is_err());

        // No digit
        assert!(validate_password("password").is_err());

        // No letter
        assert!(validate_password("12345678").is_err());
    }
}

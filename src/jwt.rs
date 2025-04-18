use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Serialize, Deserialize};
use crate::user::Role;
use std::fs;


#[derive(Serialize, Deserialize)]
pub struct Claims{
    pub subject: usize,
    pub exp: usize,
    pub role: Role
}

impl Claims{
    pub fn new(subject: usize,role: Role) -> Claims {
        Claims{
            role,
            subject,
            exp: (chrono::Utc::now().timestamp() + 3600) as usize,
        }
    }
}

fn get_validation() -> Validation{
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0;
    validation.algorithms = vec![Algorithm::RS256];
    validation
}

pub fn validate(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let validation = get_validation();
    let public_key_pem = fs::read("public_key.pem")
        .expect("public_key.pem not found");

    let decoding_key = DecodingKey::from_rsa_pem(&public_key_pem)?;

    let decode = decode::<Claims>(
        jwt,
        &decoding_key,
        &validation,
    );
    decode
}


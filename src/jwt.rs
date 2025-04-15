use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Serialize, Deserialize};
use crate::user::Role;


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
    validation.algorithms = vec![Algorithm::HS256];
    validation
}

pub fn validate(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let validation = get_validation();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECTRET should be setted");
    let decode = decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    );
    decode
}


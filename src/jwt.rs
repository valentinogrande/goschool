use jsonwebtoken::{Validation, Algorithm};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Claims{
    pub subject: usize,
    pub exp: usize,
}

impl Claims{
    pub fn new(subject: usize) -> Claims {
        Claims{
            subject,
            exp: (chrono::Utc::now().timestamp() + 3600) as usize,
        }
    }
}


pub fn get_validator() -> Validation{
    let mut validation = Validation::default();

    validation.validate_exp = true;
    validation.leeway = 0;
    validation.algorithms = vec![Algorithm::HS256];
    validation
}

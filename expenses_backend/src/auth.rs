use crate::models::User;
use jsonwebtoken::{encode,decode,Header,Validation,EncodingKey,DecodingKey,Algorithm};
use serde::{Serialize,Deserialize};
use chrono::{Utc,Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims{
    pub user_id:String,
    pub email:String,
    pub username:String,
    pub is_admin:bool,
    pub exp: i64,
}

pub fn hash_password(password:&str)->Result<String,bcrypt::BcryptError>{
    bcrypt::hash(password,bcrypt::DEFAULT_COST)
}

pub fn verify_password(password:&str,hash:&str)->Result<bool,bcrypt::BcryptError>{
    bcrypt::verify(password,hash)
}

pub fn create_jwt(user:&User,secret:&str)->Result<String,jsonwebtoken::errors::Error>{
    let expiration=Utc::now()+Duration::hours(24);

    let claims=Claims{
        user_id:user.id.to_string(),
        email:user.email.clone(),
        username:user.username.clone(),
        is_admin:user.is_admin,
        exp:expiration.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref())
    )
}

pub fn verify_jwt(token:&str,secret:&str)->Result<Claims,jsonwebtoken::errors::Error>{
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256)
    )
    .map(|data|data.claims)
}
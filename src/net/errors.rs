use std::fmt;

use actix_web::{error, HttpResponse};
use bitcoincash_addr::AddressError;
use prost::DecodeError;
use rocksdb::Error as RocksError;

use crate::crypto::errors::CryptoError;

#[derive(Debug)]
pub enum ValidationError {
    KeyType,
    Preimage,
    EmptyPayload,
    Outdated,
    ExpiredTTL,
    Crypto(CryptoError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ValidationError::KeyType => "bad key type",
            ValidationError::Preimage => "digest mismatch",
            ValidationError::EmptyPayload => "empty payload",
            ValidationError::Outdated => "metadata is outdated",
            ValidationError::ExpiredTTL => "expired TTL",
            ValidationError::Crypto(err) => return err.fmt(f),
        };
        write!(f, "{}", printable)
    }
}

impl Into<ValidationError> for CryptoError {
    fn into(self) -> ValidationError {
        ValidationError::Crypto(self)
    }
}

#[derive(Debug)]
pub enum PaymentError {
    InvalidAuth,
    Bip70Server(reqwest::Error),
    Payload,
    Decode,
    EmptyPaymentRequest,
}

impl From<PaymentError> for ServerError {
    fn from(err: PaymentError) -> Self {
        ServerError::Payment(err)
    }
}

impl From<reqwest::Error> for PaymentError {
    fn from(err: reqwest::Error) -> PaymentError {
        PaymentError::Bip70Server(err)
    }
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            PaymentError::InvalidAuth => "invalid payment token",
            PaymentError::Bip70Server(err) => return err.fmt(f),
            PaymentError::Payload => "failed fetching payload",
            PaymentError::Decode => "failed to decode invoice response",
            PaymentError::EmptyPaymentRequest => "no payment request",
        };
        write!(f, "{}", printable)
    }
}

impl error::ResponseError for PaymentError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PaymentError::InvalidAuth => HttpResponse::BadRequest(),
            _ => return HttpResponse::InternalServerError().finish(), // Don't expose internal errors
        }
        .body(self.to_string())
    }
}

#[derive(Debug)]
pub enum ServerError {
    DB(RocksError),
    Validation(ValidationError),
    Crypto(CryptoError),
    NotFound,
    MetadataDecode,
    UnsupportedSigScheme,
    Payment(PaymentError),
    Address(AddressError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ServerError::DB(err) => return err.fmt(f),
            ServerError::Crypto(err) => return err.fmt(f),
            ServerError::NotFound => "not found",
            ServerError::MetadataDecode => "metadata decoding error",
            ServerError::UnsupportedSigScheme => "signature scheme not supported",
            ServerError::Payment(err) => return err.fmt(f),
            ServerError::Validation(err) => return err.fmt(f),
            ServerError::Address(err) => return err.fmt(f),
        };
        write!(f, "{}", printable)
    }
}

impl From<AddressError> for ServerError {
    fn from(err: AddressError) -> Self {
        ServerError::Address(err)
    }
}

impl From<CryptoError> for ServerError {
    fn from(err: CryptoError) -> Self {
        ServerError::Crypto(err)
    }
}

impl From<DecodeError> for ServerError {
    fn from(_: DecodeError) -> Self {
        ServerError::MetadataDecode
    }
}

impl From<RocksError> for ServerError {
    fn from(err: RocksError) -> Self {
        ServerError::DB(err)
    }
}

impl From<ValidationError> for ServerError {
    fn from(err: ValidationError) -> ServerError {
        ServerError::Validation(err)
    }
}

impl error::ResponseError for CryptoError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CryptoError::PubkeyDeserialization => HttpResponse::BadRequest(),
            CryptoError::SigDeserialization => HttpResponse::BadRequest(),
            CryptoError::Verification => HttpResponse::BadRequest(),
        }
        .body(self.to_string())
    }
}

impl error::ResponseError for ValidationError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ValidationError::Crypto(err_inner) => return err_inner.error_response(),
            ValidationError::EmptyPayload => HttpResponse::BadRequest(),
            ValidationError::KeyType => HttpResponse::BadRequest(),
            ValidationError::Preimage => HttpResponse::BadRequest(),
            ValidationError::Outdated => HttpResponse::BadRequest(),
            ValidationError::ExpiredTTL => HttpResponse::BadRequest(),
        }
        .body(self.to_string())
    }
}

impl error::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::Validation(err) => err.error_response(),
            // Do not yield sensitive information to clients
            ServerError::DB(_) => HttpResponse::InternalServerError().finish(), // Don't expose internal errors
            ServerError::NotFound => HttpResponse::NotFound().body(self.to_string()),
            ServerError::MetadataDecode => HttpResponse::BadRequest().body(self.to_string()),
            ServerError::UnsupportedSigScheme => HttpResponse::BadRequest().body(self.to_string()),
            ServerError::Crypto(err) => err.error_response(),
            ServerError::Payment(err) => err.error_response(),
            ServerError::Address(err) => HttpResponse::BadRequest().body(err.to_string()),
        }
    }
}

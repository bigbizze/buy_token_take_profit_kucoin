use crate::error::error::other_err::{KucoinApiError, KucoinClientError, KucoinErrorKind, MiscError};

pub mod other_err {
    use kucoin_rs_custom::kucoin::error::APIError;

    #[derive(thiserror::Error, Debug)]
    #[error("{:?}", msg)]
    pub struct MiscError {
        msg: String
    }

    impl From<String> for MiscError {
        fn from(msg: String) -> Self {
            MiscError { msg }
        }
    }

    impl MiscError {
        pub fn get_fmt_error(&mut self) -> String {
            format!("{}", self.msg)
        }
    }

    #[derive(Debug)]
    pub struct KucoinApiError {
        e: kucoin_rs_custom::kucoin::error::APIError
    }

    impl std::fmt::Display for KucoinApiError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let fmt = match &self.e {
                APIError::Serde(e) => format!("{}", e),
                APIError::Websocket(e) => format!("{}", e),
                APIError::HTTP(e) => format!("{}", e),
                APIError::Other(e) => format!("{}", e),
            };
            write!(f, "{}", fmt)
        }
    }

    impl std::error::Error for KucoinApiError {}

    impl From<APIError> for KucoinApiError {
        fn from(e: APIError) -> Self {
            KucoinApiError { e }
        }
    }

    #[derive(Debug)]
    pub struct KucoinClientError {
        e: kucoin_rs_custom::failure::Error
    }

    impl std::fmt::Display for KucoinClientError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.e)
        }
    }

    impl std::error::Error for KucoinClientError {}

    impl From<kucoin_rs_custom::failure::Error> for KucoinClientError {
        fn from(e: kucoin_rs_custom::failure::Error) -> Self {
            KucoinClientError { e }
        }
    }

    pub enum KucoinErrorKind {
        Api(APIError),
        Client(kucoin_rs_custom::failure::Error),
    }

    impl Into<KucoinErrorKind> for APIError {
        fn into(self) -> KucoinErrorKind {
            KucoinErrorKind::Api(self)
        }
    }

    impl Into<KucoinErrorKind> for kucoin_rs_custom::failure::Error {
        fn into(self) -> KucoinErrorKind {
            KucoinErrorKind::Client(self)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MintError {
    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    KucoinApiError(#[from] KucoinApiError),

    #[error(transparent)]
    KucoinClientError(#[from] KucoinClientError),

    #[error(transparent)]
    Other(#[from] MiscError),

    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

impl MintError {
    pub fn from_str(msg: String) -> Self {
        MintError::Other(MiscError::from(msg))
    }
    pub fn from_kucoin_err(e: KucoinErrorKind) -> Self {
        match e {
            KucoinErrorKind::Api(e) => MintError::KucoinApiError(KucoinApiError::from(e)),
            KucoinErrorKind::Client(e) => MintError::KucoinClientError(KucoinClientError::from(e))
        }
    }
    pub fn get_fmt_error(&mut self) -> String {
        match self {
            MintError::Serde(e) => format!("{}", e),
            MintError::IoError(e) => format!("{}", e),
            MintError::ParseFloatError(e) => format!("{}", e),
            MintError::Other(e) => format!("{}", e.get_fmt_error()),
            MintError::ParseIntError(e) => format!("{}", e),
            MintError::KucoinApiError(e) => format!("{}", e),
            MintError::KucoinClientError(e) => format!("{}", e),
            MintError::SystemTimeError(e) => format!("{}", e)
        }
    }
}

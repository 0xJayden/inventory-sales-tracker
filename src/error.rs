use std::env::VarError;

#[derive(Clone, Debug)]
pub enum Errorr {
    ApiError,
}

impl From<sqlx::Error> for Errorr {
    fn from(error: sqlx::Error) -> Errorr {
        dbg!(error);

        Errorr::ApiError
    }
}

impl From<VarError> for Errorr {
    fn from(error: VarError) -> Errorr {
        dbg!(error);

        Errorr::ApiError
    }
}

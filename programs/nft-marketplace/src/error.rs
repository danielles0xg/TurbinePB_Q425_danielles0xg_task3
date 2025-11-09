use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Admin only")]
    Unauthorized,

    #[msg("Basis points < 10_000")]
    FeeTooHigh,

}

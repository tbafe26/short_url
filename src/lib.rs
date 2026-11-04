//! 숫자형 단축 URL 서비스의 공용 라이브러리.

pub mod dto;
pub mod short_number;
pub mod validation;

#[cfg(feature = "server")]
pub mod api;
#[cfg(feature = "server")]
pub mod links;

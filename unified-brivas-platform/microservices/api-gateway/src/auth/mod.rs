//! Authentication Module

mod oauth;

pub use oauth::{OAuthProvider, OAuthProviderRegistry, OAuthTokenResponse, OAuthUserInfo, OAuthError};

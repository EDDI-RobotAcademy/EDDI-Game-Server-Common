use dotenv::dotenv;
use std::env;

pub struct EnvDetector;

impl EnvDetector {
    pub fn get_var(key: &str) -> Option<String> {
        dotenv().ok();

        match env::var(key) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub fn get_host() -> Option<String> {
        Self::get_var("HOST")
    }

    pub fn get_port() -> Option<String> {
        Self::get_var("PORT")
    }

    pub fn get_bind_address() -> Option<String> {
        match (Self::get_host(), Self::get_port()) {
            (Some(host), Some(port)) => Some(format!("{}:{}", host, port)),
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_port() {
        env::set_var("PORT", "1234");

        assert_eq!(EnvDetector::get_port(), Some("1234".to_string()));
    }

    #[test]
    fn test_get_host() {
        env::set_var("HOST", "0.0.0.0");
        assert_eq!(EnvDetector::get_host(), Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_get_bind_address() {
        env::remove_var("HOST");
        env::remove_var("PORT");
        env::set_var("HOST", "0.0.0.0");
        env::set_var("PORT", "1234");

        assert_eq!(
            EnvDetector::get_bind_address(),
            Some("0.0.0.0:1234".to_string())
        );
    }


    #[test]
    fn test_get_bind_address_missing() {
        env::remove_var("HOST");
        env::remove_var("PORT");
        assert_eq!(EnvDetector::get_bind_address(), Some("0.0.0.0:1234".to_string()));
    }
}
use shared::error::AppError;

macro_rules! define_value {
    ($name:ident, $rules:meta) => {
        #[derive(Debug, Clone, PartialEq, Eq, garde::Validate)]
        pub struct $name(#[garde($rules)] String);
        impl $name {
            pub fn into_inner(self) -> String {
                self.0
            }
        }
        impl AsRef<String> for $name {
            fn as_ref(&self) -> &String {
                &self.0
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
        impl std::str::FromStr for $name {
            type Err = AppError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let res = Self(value.to_string());
                garde::Validate::validate(&res, &())?;
                Ok(res)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_value!(UserName, length(min = 1));
define_value!(UserEmail, email);
define_value!(BookTitle, length(min = 1));
define_value!(BookAuthor, length(min = 1));
define_value!(BookIsbn, length(min = 1));
define_value!(BookDescription, skip);

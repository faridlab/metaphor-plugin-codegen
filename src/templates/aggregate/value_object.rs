//! {{PascalCaseEntity}} Value Object Implementation
//!
//! This is a value object representing {{PascalCaseEntity}}.
//! Value objects are immutable and defined by their attributes.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// {{PascalCaseEntity}} Value Object
///
/// This value object represents {{PascalCaseEntity}} in the domain.
/// Value objects are immutable and identified by their attributes, not by an ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}} {
    // TODO: Add value object fields here
    // Examples for different types of value objects:

    // For Email Address:
    // pub value: String,
    // pub is_verified: bool,

    // For Money:
    // pub currency: String,
    // pub amount: i64,

    // For Address:
    // pub street: String,
    // pub city: String,
    // pub state: String,
    // pub postal_code: String,
    // pub country: String,

    // For Range:
    // pub min_value: i64,
    // pub max_value: i64,

    // TODO: Replace this placeholder with actual fields
    pub value: String,
}

impl {{PascalCaseEntity}} {
    /// Create a new {{PascalCaseEntity}} value object
    ///
    /// # Arguments
    ///
    /// * `value` - The value for this value object
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the value object or a validation error
    pub fn new(value: String) -> Result<Self, {{PascalCaseEntity}}Error> {
        // TODO: Validate the value object
        // Example validation:
        // if value.is_empty() {
        //     return Err({{PascalCaseEntity}}Error::InvalidValue("Value cannot be empty".to_string()));
        // }

        // TODO: Add business rule validation
        // Example for email:
        // if !value.contains('@') {
        //     return Err({{PascalCaseEntity}}Error::InvalidValue("Invalid email format".to_string()));
        // }

        // Example for money:
        // if amount < 0 {
        //     return Err({{PascalCaseEntity}}Error::InvalidValue("Amount cannot be negative".to_string()));
        // }

        Ok(Self { value })
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    // TODO: Add helper methods for value object
    // Example for Email:
    // pub fn is_verified(&self) -> bool {
    //     self.is_verified
    // }
    //
    // pub fn domain(&self) -> &str {
    //     self.value.split('@').nth(1).unwrap_or("")
    // }

    // Example for Money:
    // pub fn amount(&self) -> i64 {
    //     self.amount
    // }
    //
    // pub fn currency(&self) -> &str {
    //     &self.currency
    // }
    //
    // pub fn is_positive(&self) -> bool {
    //     self.amount > 0
    // }
    //
    // pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
    //     if self.currency != other.currency {
    //         return Err(MoneyError::CurrencyMismatch);
    //     }
    //     Money::new(self.currency.clone(), self.amount + other.amount)
    // }

    // Example for Address:
    // pub fn full_address(&self) -> String {
    //     format!("{}, {}, {} {}", self.street, self.city, self.state, self.postal_code)
    // }
    //
    // pub fn is_in_country(&self, country: &str) -> bool {
    //     self.country == country
    // }
}

// TODO: Implement Display trait for value object
impl Display for {{PascalCaseEntity}} {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Customize display format
        write!(f, "{}", self.value)

        // Example for Email:
        // write!(f, "{}", self.value)

        // Example for Money:
        // write!(f, "{} {}", self.amount, self.currency)

        // Example for Address:
        // write!(f, "{}, {}, {} {}", self.street, self.city, self.state, self.postal_code)
    }
}

// TODO: Implement validation traits
// impl Validate for {{PascalCaseEntity}} {
//     fn validate(&self) -> Result<(), ValidationError> {
//         if self.value.is_empty() {
//             return Err(ValidationError::new("value", "Value cannot be empty"));
//         }
//         Ok(())
//     }
// }

/// {{PascalCaseEntity}} value object errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum {{PascalCaseEntity}}Error {
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Currency mismatch")]
    CurrencyMismatch,
}

// TODO: Add conversion traits if needed
// Example: From<String> for Email
// impl TryFrom<String> for {{PascalCaseEntity}} {
//     type Error = {{PascalCaseEntity}}Error;
//
//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         Self::new(value)
//     }
// }
//
// impl TryFrom<&str> for {{PascalCaseEntity}} {
//     type Error = {{PascalCaseEntity}}Error;
//
//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         Self::new(value.to_string())
//     }
// }

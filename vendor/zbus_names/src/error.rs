use static_assertions::assert_impl_all;
use std::{convert::Infallible, error, fmt};
use zvariant::Error as VariantError;

/// The error type for `zbus_names`.
///
/// The various errors that can be reported by this crate.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    Variant(VariantError),
    /// Invalid bus name. The strings describe why the bus name is neither a valid unique nor
    /// well-known name, respectively.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidBusName(String, String),
    /// Invalid well-known bus name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidWellKnownName(String),
    /// Invalid unique bus name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidUniqueName(String),
    /// Invalid interface name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidInterfaceName(String),
    /// Invalid member (method or signal) name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidMemberName(String),
    /// Invalid property name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidPropertyName(String),
    /// Invalid error name.
    #[deprecated(
        since = "4.1.0",
        note = "This variant is no longer returned from any of our API.\
                Use `Error::InvalidName` instead."
    )]
    InvalidErrorName(String),
    /// An invalid name.
    InvalidName(&'static str),
    /// Invalid conversion from name type `from` to name type `to`.
    InvalidNameConversion {
        from: &'static str,
        to: &'static str,
    },
}

assert_impl_all!(Error: Send, Sync, Unpin);

impl PartialEq for Error {
    #[allow(deprecated)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidBusName(_, _), Self::InvalidBusName(_, _)) => true,
            (Self::InvalidWellKnownName(_), Self::InvalidWellKnownName(_)) => true,
            (Self::InvalidUniqueName(_), Self::InvalidUniqueName(_)) => true,
            (Self::InvalidInterfaceName(_), Self::InvalidInterfaceName(_)) => true,
            (Self::InvalidMemberName(_), Self::InvalidMemberName(_)) => true,
            (Self::InvalidPropertyName(_), Self::InvalidPropertyName(_)) => true,
            (Self::InvalidErrorName(_), Self::InvalidErrorName(_)) => true,
            (Self::InvalidName(_), Self::InvalidName(_)) => true,
            (Self::InvalidNameConversion { .. }, Self::InvalidNameConversion { .. }) => true,
            (Self::Variant(s), Self::Variant(o)) => s == o,
            (_, _) => false,
        }
    }
}

impl error::Error for Error {
    #[allow(deprecated)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::InvalidBusName(_, _) => None,
            Error::InvalidWellKnownName(_) => None,
            Error::InvalidUniqueName(_) => None,
            Error::InvalidInterfaceName(_) => None,
            Error::InvalidErrorName(_) => None,
            Error::InvalidMemberName(_) => None,
            Error::InvalidPropertyName(_) => None,
            Error::InvalidName(_) => None,
            Error::InvalidNameConversion { .. } => None,
            Error::Variant(e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Variant(e) => write!(f, "{e}"),
            Error::InvalidBusName(unique_err, well_known_err) => {
                write!(
                    f,
                    "Neither a valid unique ({unique_err}) nor a well-known ({well_known_err}) bus name"
                )
            }
            Error::InvalidWellKnownName(s) => write!(f, "Invalid well-known bus name: {s}"),
            Error::InvalidUniqueName(s) => write!(f, "Invalid unique bus name: {s}"),
            Error::InvalidInterfaceName(s) => write!(f, "Invalid interface or error name: {s}"),
            Error::InvalidErrorName(s) => write!(f, "Invalid interface or error name: {s}"),
            Error::InvalidMemberName(s) => write!(f, "Invalid method or signal name: {s}"),
            Error::InvalidPropertyName(s) => write!(f, "Invalid property name: {s}"),
            Error::InvalidName(s) => write!(f, "{s}"),
            Error::InvalidNameConversion { from, to } => {
                write!(f, "Invalid conversion from `{from}` to `{to}`")
            }
        }
    }
}

impl From<VariantError> for Error {
    fn from(val: VariantError) -> Self {
        Error::Variant(val)
    }
}

impl From<Infallible> for Error {
    fn from(i: Infallible) -> Self {
        match i {}
    }
}

/// Alias for a `Result` with the error type `zbus_names::Error`.
pub type Result<T> = std::result::Result<T, Error>;

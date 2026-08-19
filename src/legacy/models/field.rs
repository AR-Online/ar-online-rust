//! The three answers a legacy field gives.

use serde::{Deserialize, Deserializer};

/// A field of the old contract that may not be there.
///
/// The gateway says "this has not happened yet" in four different ways -- `""`,
/// `null`, a key that vanishes from the response, and `{}` -- and sometimes two
/// of them in the same body. Three of those map onto Rust without help: `""` is
/// a `String`, `{}` is an empty map, and a key that is always present but
/// nullable is an `Option`.
///
/// The fourth does not. `Option<T>` collapses "the key came as `null`" and "the
/// key was not in the response" into the same `None`, and those are different
/// facts on this API: the e-mail route sends `dateReading: null`, while the
/// `WhatsApp` route simply does not send `dateDelivery` at all. Whoever is
/// debugging a stuck notification needs to tell one from the other, and
/// normalizing them would throw away the fidelity this whole area exists to
/// give.
///
/// So the rule across the legacy models is:
///
/// | on the wire | in the struct |
/// |---|---|
/// | always present, `""` when it has not happened | `String` |
/// | always present, `null` when it has not happened | `Option<T>` |
/// | the key vanishes when it has not happened | `LegacyField<T>` |
///
/// `LegacyField` covers all three states, so a route that starts sending
/// `null` where it used to drop the key does not break anyone.
///
/// ```
/// use aronline::legacy::LegacyField;
///
/// let vanished: LegacyField<String> = LegacyField::Missing;
///
/// assert!(vanished.is_missing());
/// assert_eq!(vanished.value(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyField<T> {
    /// The key was not in the response.
    Missing,
    /// The key was there, carrying `null`.
    Null,
    /// The key was there, carrying a value.
    Present(T),
}

impl<T> LegacyField<T> {
    /// The value, when there is one. `None` for both flavours of absence.
    #[must_use]
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }

    /// The value, taking ownership.
    #[must_use]
    pub fn into_value(self) -> Option<T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }

    /// Whether the key was absent from the response.
    #[must_use]
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Whether the key was there carrying `null`.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

// Missing, and not Null: serde reaches for the default exactly when the key was
// not in the response, which is what `Missing` means.
//
// Written out instead of derived, and the lint is off for it: `#[derive(Default)]`
// on a generic enum bolts a `T: Default` bound onto the impl, so
// `LegacyField<T>` would stop having a default for any T that has none -- a
// narrower type for no gain, since the default here never looks at T.
#[allow(clippy::derivable_impls)]
impl<T> Default for LegacyField<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for LegacyField<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // This only runs when the key IS in the response -- a missing key never
        // reaches a Deserialize impl, it takes the `#[serde(default)]` path
        // above. So `null` here is an explicit null, and nothing else.
        Ok(Option::<T>::deserialize(deserializer)?.map_or(Self::Null, Self::Present))
    }
}

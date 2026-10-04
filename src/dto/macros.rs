//! Macros for integer-encoded API values.

macro_rules! int_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $value:literal ),+ $(,)?
        }
        default = $default:expr;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// A value this client does not know about.
            Unknown(i64),
        }

        impl $name {
            pub const fn value(self) -> i64 {
                match self {
                    $( Self::$variant => $value, )+
                    Self::Unknown(value) => value,
                }
            }

            pub const fn from_value(value: i64) -> Self {
                match value {
                    $( $value => Self::$variant, )+
                    other => Self::Unknown(other),
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $default
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.value())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <i64 as serde::Deserialize>::deserialize(deserializer).map(Self::from_value)
            }
        }
    };
}

/// Bit flags serialized as a plain integer (bitflags' own serde support uses names).
macro_rules! int_flags {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident: $repr:ty {
            $( $(#[$($fmeta:tt)*])* const $flag:ident = $value:expr; )*
        }
    ) => {
        bitflags::bitflags! {
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            $vis struct $name: $repr {
                $( $(#[$($fmeta)*])* const $flag = $value; )*
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.bits(), serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$repr as serde::Deserialize>::deserialize(deserializer).map(Self::from_bits_retain)
            }
        }
    };
}

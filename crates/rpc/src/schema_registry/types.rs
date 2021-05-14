use std::convert::TryFrom;

use async_graphql::Enum;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use super::filter_operator;
use super::logic_operator;
use super::schema_type;
use super::search_for;

macro_rules! rpc_enum {
	(
        $name: ident,
        $inner: path,
        $inner_field: ident,
        $display: literal,
        $sql: literal,
        [ $($variant: ident),* ]
    ) => {
        #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, Enum, Display)]
        #[sqlx(type_name = $sql, rename_all = "lowercase")]
        pub enum $name {
            $($variant),*
        }

        impl From<$inner> for $name {
            fn from(op: $inner) -> Self {
                match op {
                    $(<$inner>::$variant => Self::$variant),*
                }
            }
        }

        impl From<$name> for super::$name {
            fn from(op: $name) -> Self {
                Self {
                    $inner_field: match op {
                        $(<$name>::$variant => <$inner>::$variant.into()),*
                    },
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => Ok(Self::$variant),
                    )*
                    invalid => Err(anyhow::anyhow!("Invalid {}: {}", $display, invalid)),
                }
            }
        }

        impl From<$name> for i32 {
            fn from(o: $name) -> i32 {
                let o: super::$name = o.into();
                o.$inner_field
            }
        }

        impl TryFrom<super::$name> for $name {
            type Error = anyhow::Error;

            fn try_from(variant: super::$name) -> Result<Self, Self::Error> {
                if false { unreachable!() } //In case there are no variants
                $( if variant.$inner_field == { let v: i32 = <$inner>::$variant.into(); v } { return Ok(Self::$variant) })*
                else {  Err(anyhow::anyhow!(concat!("Invalid ", $display))) }
            }
        }
	};
}

rpc_enum! {
    SchemaType,
    schema_type::Type,
    schema_type,
    "schema type",
    "schema_type_enum",
    [
        DocumentStorage,
        Timeseries
    ]
}

rpc_enum! {
    SearchFor,
    search_for::Direction,
    search_for,
    "relation direction",
    "search_for_enum",
    [
        Parents,
        Children
    ]
}

rpc_enum! {
    FilterOperator,
    filter_operator::Operator,
    operator,
    "filter operator",
    "filter_operator_enum",
    [
        Equals
    ]
}

rpc_enum! {
    LogicOperator,
    logic_operator::Operator,
    operator,
    "logic operator",
    "logic_operator_enum",
    [
        And,
        Or
    ]
}

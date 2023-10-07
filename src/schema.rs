// @generated automatically by Diesel CLI.

use core::fmt;
use diesel_derive_enum::DbEnum;

#[derive(Debug, DbEnum)]
pub enum Category {
    #[db_rename = "dev"]
    Dev,
    #[db_rename = "infra"]
    Infra,
    #[db_rename = "devinfra"]
    DevInfra,
    #[db_rename = "marketing"]
    Marketing,
    #[db_rename = "common"]
    Common,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Dev => write!(f, "Dev"),
            Category::Infra => write!(f, "Infra"),
            Category::DevInfra => write!(f, "Dev/Infra"),
            Category::Marketing => write!(f, "Marketing"),
            Category::Common => write!(f, "Tronc Commun"),
        }
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::schema::CategoryMapping;

    course (id) {
        id -> Integer,
        date -> Date,
        #[max_length = 9]
        category -> CategoryMapping,
        start -> Time,
        end -> Time,
        #[max_length = 255]
        subject -> Varchar,
        #[max_length = 255]
        teacher -> Varchar,
        #[max_length = 255]
        classroom -> Varchar,
        remote -> Bool,
        bts -> Bool,
    }
}

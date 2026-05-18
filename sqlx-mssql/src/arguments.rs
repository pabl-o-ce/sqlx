use std::fmt::{self, Write};

use crate::database::MssqlArgumentValue;
use crate::encode::Encode;
use crate::types::Type;
use crate::Mssql;
pub(crate) use sqlx_core::arguments::*;
use sqlx_core::error::BoxDynError;

/// Implementation of [`Arguments`] for MSSQL.
#[derive(Debug, Default, Clone)]
pub struct MssqlArguments {
    pub(crate) values: Vec<MssqlArgumentValue>,
}

impl MssqlArguments {
    pub(crate) fn add<'q, T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: Encode<'q, Mssql> + Type<Mssql>,
    {
        // Detect "encoder reported null without pushing" by comparing len()
        // before and after — the previous check used `last() is Null`, which
        // mistakenly dropped a Null bind whenever the previous parameter was
        // also Null (e.g. `bind(None).bind(None)`).
        let before = self.values.len();
        let is_null = value.encode(&mut self.values)?;
        if is_null.is_null() && self.values.len() == before {
            self.values.push(MssqlArgumentValue::Null);
        }
        Ok(())
    }
}

impl Arguments for MssqlArguments {
    type Database = Mssql;

    fn reserve(&mut self, len: usize, _size: usize) {
        self.values.reserve(len);
    }

    fn add<'t, T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: Encode<'t, Self::Database> + Type<Self::Database>,
    {
        self.add(value)
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn format_placeholder<W: Write>(&self, writer: &mut W) -> fmt::Result {
        // MSSQL uses @p1, @p2, ... for parameterized queries.
        // This is called after the bind is added, so len() is the correct 1-based index.
        write!(writer, "@p{}", self.values.len())
    }
}

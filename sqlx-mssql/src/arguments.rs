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
        let is_null = value.encode(&mut self.values)?;
        if is_null.is_null() {
            // If the encoder signaled null but didn't push a value, push a Null
            if self.values.last().map_or(true, |v| !matches!(v, MssqlArgumentValue::Null)) {
                self.values.push(MssqlArgumentValue::Null);
            }
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
}

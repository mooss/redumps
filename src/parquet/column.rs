use std::sync::Arc;

use arrow::array::{ArrayBuilder, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field};

use crate::parquet::json_to_parquet::ParquetRow;

/// Represents a column in the Parquet output, as a tuple of:
///  1. Arrow array builder (accumulates column data).
///  2. Arrow field.
///  3. Value extractor.
pub enum Column {
    /// String column.
    String(StringBuilder, Field, fn(&ParquetRow) -> Option<&str>),
    /// 64 bit integer column.
    Int64(Int64Builder, Field, fn(&ParquetRow) -> Option<i64>),
    /// Boolean column.
    Boolean(BooleanBuilder, Field, fn(&ParquetRow) -> Option<bool>),
    /// 64 bit float column.
    Float64(Float64Builder, Field, fn(&ParquetRow) -> Option<f64>),
}

impl Column {
    /// Construct a new string column.
    pub fn new_string(name: &str, f: fn(&ParquetRow) -> Option<&str>) -> Self {
        Self::String(
            StringBuilder::new(),
            Field::new(name, DataType::Utf8, true),
            f,
        )
    }

    /// Construct a new 64 bit integer column.
    pub fn new_int64(name: &str, f: fn(&ParquetRow) -> Option<i64>) -> Self {
        Self::Int64(
            Int64Builder::new(),
            Field::new(name, DataType::Int64, true),
            f,
        )
    }

    /// Construct a new boolean column.
    pub fn new_boolean(name: &str, f: fn(&ParquetRow) -> Option<bool>) -> Self {
        Self::Boolean(
            BooleanBuilder::new(),
            Field::new(name, DataType::Boolean, true),
            f,
        )
    }

    /// Construct a new 64 bit float column.
    pub fn new_float64(name: &str, f: fn(&ParquetRow) -> Option<f64>) -> Self {
        Self::Float64(
            Float64Builder::new(),
            Field::new(name, DataType::Float64, true),
            f,
        )
    }

    /// Return the Arrow field associated with this column.
    pub fn field(&self) -> &Field {
        match self {
            Self::String(_, field, _) => field,
            Self::Int64(_, field, _) => field,
            Self::Boolean(_, field, _) => field,
            Self::Float64(_, field, _) => field,
        }
    }

    /// Append a value from `row` to the underlying builder.
    pub fn append(&mut self, row: &ParquetRow) {
        match self {
            Self::String(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Int64(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Boolean(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Float64(builder, _, extractor) => builder.append_option(extractor(row)),
        }
    }

    /// Finalise the builder and return the resulting Arrow array.
    pub fn finish(&mut self) -> Arc<dyn arrow::array::Array> {
        match self {
            Self::String(builder, _, _) => Arc::new(builder.finish()),
            Self::Int64(builder, _, _) => Arc::new(builder.finish()),
            Self::Boolean(builder, _, _) => Arc::new(builder.finish()),
            Self::Float64(builder, _, _) => Arc::new(builder.finish()),
        }
    }

    /// Reset the builder to its initial empty state.
    pub fn reset(&mut self) {
        match self {
            Self::String(builder, _, _) => *builder = StringBuilder::new(),
            Self::Int64(builder, _, _) => *builder = Int64Builder::new(),
            Self::Boolean(builder, _, _) => *builder = BooleanBuilder::new(),
            Self::Float64(builder, _, _) => *builder = Float64Builder::new(),
        }
    }

    /// Current number of appended values in the column.
    pub fn len(&self) -> usize {
        match self {
            Self::String(builder, _, _) => builder.len(),
            Self::Int64(builder, _, _) => builder.len(),
            Self::Boolean(builder, _, _) => builder.len(),
            Self::Float64(builder, _, _) => builder.len(),
        }
    }

    /// Whether the column currently holds no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

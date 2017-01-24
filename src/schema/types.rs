use basic::{Type as PhysicalType, Repetition, LogicalType};
use errors::Result;
use errors::ParquetError;

enum TypeKind {
  PRIMITIVE,
  GROUP
}

/// A trait for a logical schema type. Structs who implement
/// this needs to implement the `get_basic_info()` function.
trait Type {
  fn is_primitive(&self) -> bool {
    return match self.get_basic_info().kind {
      TypeKind::PRIMITIVE => true,
      _ => false
    }
  }

  fn is_group(&self) -> bool {
    return match self.get_basic_info().kind {
      TypeKind::GROUP => true,
      _ => false
    }
  }

  fn name(&self) -> &str {
    return &self.get_basic_info().name;
  }

  fn repetition(&self) -> Repetition {
    return self.get_basic_info().repetition;
  }

  fn logical_type(&self) -> LogicalType {
    return self.get_basic_info().logical_type;
  }

  fn id(&self) -> i32 {
    return self.get_basic_info().id;
  }

  fn get_basic_info(&self) -> &BasicTypeInfo;
}

/// Basic type info. This contains information such as the name of the type,
/// the repetition level, the logical type and the kind of the type (group, primitive).
struct BasicTypeInfo {
  kind: TypeKind,
  name: &'static str,
  repetition: Repetition,
  logical_type: LogicalType,
  id: i32
}

/// Metadata for a decimal type (scale, precision).
struct DecimalMetadata {
  scale: i32,
  precision: i32
}

/// Representation of primitive types
struct PrimitiveType {
  basic_info: BasicTypeInfo,
  physical_type: PhysicalType,
  type_length: i32,
  decimal_metadata: Option<DecimalMetadata>
}

impl PrimitiveType {
  pub fn new(name: &'static str, repetition: Repetition, physical_type: PhysicalType,
             logical_type: LogicalType, length: i32,
             precision: i32, scale: i32, id: i32) -> Result<Self> {
    let mut decimal_metadata = None;
    let basic_info = BasicTypeInfo{
      kind: TypeKind::PRIMITIVE, name: name, repetition: repetition,
      logical_type: logical_type, id: id};

    match logical_type {
      LogicalType::NONE => {
        decimal_metadata = Some(DecimalMetadata{precision, scale});
      },
      LogicalType::UTF8 | LogicalType::BSON | LogicalType::JSON => {
        if physical_type != PhysicalType::BYTE_ARRAY {
          return Err(ParquetError::Schema(
            format!("{} can only annotate BYTE_ARRAY fields", logical_type)))
        }
      },
      LogicalType::DECIMAL => {
        match physical_type {
          PhysicalType::INT32 | PhysicalType::INT64 | PhysicalType::BYTE_ARRAY |
          PhysicalType::FIXED_LEN_BYTE_ARRAY => (),
          _ => {
            return Err(ParquetError::Schema(
            format!("DECIMAL can only annotate INT32, INT64, BYTE_ARRAY and FIXED")))
          }
        };
        if precision < 0 {
          // TODO: how can we make ParquetError::Schema to accept `&'static str`?
          return Err(ParquetError::Schema(
            format!("Invalid DECIMAL precision: {}", precision)))
        }
        if scale < 0 {
          return Err(ParquetError::Schema(
            format!("Invalid DECIMAL scale: {}", scale)))
        }
        if scale > precision {
          return Err(ParquetError::Schema(
            format!("Invalid DECIMAL: scale {} cannot be greater than precision {}",
                    scale, precision)))
        }
        decimal_metadata = Some(DecimalMetadata{precision, scale})
      }
      LogicalType::DATE | LogicalType::TIME_MILLIS | LogicalType::UINT_8 |
      LogicalType::UINT_16 | LogicalType::UINT_32 |
      LogicalType::INT_8 | LogicalType::INT_16 | LogicalType::INT_32 => {
        if physical_type != PhysicalType::INT32 {
          return Err(ParquetError::Schema(
            format!("{} can only annotate INT32", logical_type)))
        }
      }
      LogicalType::TIME_MICROS | LogicalType::TIMESTAMP_MILLIS |
      LogicalType::TIMESTAMP_MICROS | LogicalType::UINT_64 | LogicalType::INT_64 => {
        if physical_type != PhysicalType::INT64 {
          return Err(ParquetError::Schema(
            format!("{} can only annotate INT64", logical_type)))
        }
      }
      LogicalType::INTERVAL => {
        if physical_type != PhysicalType::FIXED_LEN_BYTE_ARRAY || length != 12 {
          return Err(ParquetError::Schema(String::from("INTERVAL can only annotate FIXED(12)")))
        }
      }
      LogicalType::ENUM => {
        if physical_type != PhysicalType::BYTE_ARRAY {
          return Err(ParquetError::Schema(
            String::from("ENUM can only annotate BYTE_ARRAY fields")))
        }
      }
      _ => {
        return Err(ParquetError::Schema(
          format!("{} cannot be applied to a primitive type", logical_type)))
      }
    };
    if physical_type == PhysicalType::FIXED_LEN_BYTE_ARRAY && length < 0 {
      return Err(ParquetError::Schema(
        format!("Invalid FIXED_LEN_BYTE_ARRAY length: {}", length)
      ))
    }
    Ok(PrimitiveType{
      basic_info: basic_info,
      physical_type: physical_type,
      type_length: length,
      decimal_metadata: decimal_metadata
    })
  }
}

impl Type for PrimitiveType {
  fn get_basic_info(&self) -> &BasicTypeInfo {
    return &self.basic_info;
  }
}

/// Representation of group types
struct GroupType {
  basic_info: BasicTypeInfo,
  fields: Vec<Box<Type>>
}

impl GroupType {
  pub fn new(name: &'static str, repetition: Repetition, physical_type: PhysicalType,
             logical_type: LogicalType, length: i32,
             fields: Vec<Box<Type>>, id: i32) -> Result<Self> {
    let basic_info = BasicTypeInfo{
      kind: TypeKind::PRIMITIVE, name: name, repetition: repetition,
      logical_type: logical_type, id: id};
    Ok(GroupType{
      basic_info: basic_info,
      fields: fields
    })
  }

  pub fn num_fields(&self) -> usize {
    return self.fields.len()
  }

  pub fn field(&mut self, index: usize) -> &mut Box<Type> {
    return &mut self.fields[index]
  }
}

impl Type for GroupType {
  fn get_basic_info(&self) -> &BasicTypeInfo {
    return &self.basic_info;
  }
}

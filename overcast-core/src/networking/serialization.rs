
/// I would like a crate that allow this, 
/// but could not find one that serialize into a slice
/// and returns the amount of bytes wrote.
pub trait Serializable: Sized {
    /// The maximum binary size the object will take.
    /// For structs, this is always the serialized size.
    /// For enums, it is the biggest serialized size among variants.
    const MAX_BIN_SIZE: usize;
    /// Writes the object into the target slice,
    /// and returns the amount of bytes written.
    /// 
    /// This will panic if the serialized size if bigger than the provided slice,
    /// And it is the caller responsability to ensure the slice is big enough.
    /// The size needed is given by MAX_BIN_SIZE.
    fn serialize(&self, into: &mut [u8]) -> usize;
    /// Builds the object from the given slice.
    /// 
    /// Returns an error if there is not enough provided bytes.
    /// This will panic if the slice does not contains enough bytes,
    /// And it is the caller responsability to ensure the slice is big enough.
    /// The size needed is given by MAX_BIN_SIZE.
    fn deserialize(from: &[u8]) -> Self;
}

macro_rules! impl_serializable_for_primitive {
    ($ty:ty, $byte_count:expr) => {
        impl Serializable for $ty {
            const MAX_BIN_SIZE: usize = $byte_count;
            fn serialize(&self, into: &mut [u8]) -> usize {
                let bytes = self.to_be_bytes();
                for i in 0..$byte_count {
                    into[i] = bytes[i]
                }
                Self::MAX_BIN_SIZE
            }
            fn deserialize(from: &[u8]) -> Self {
                let mut bytes = [0u8; $byte_count];
                for i in 0..$byte_count {
                    bytes[i] = from[i];
                }
                <$ty>::from_be_bytes(bytes)
            }
        }
    };
}

impl_serializable_for_primitive!(u8, 1);
impl_serializable_for_primitive!(u16, 2);
impl_serializable_for_primitive!(u32, 4);
impl_serializable_for_primitive!(u64, 8);
impl_serializable_for_primitive!(i8, 1);
impl_serializable_for_primitive!(i16, 2);
impl_serializable_for_primitive!(i32, 4);
impl_serializable_for_primitive!(i64, 8);
impl_serializable_for_primitive!(f32, 4);
impl_serializable_for_primitive!(f64, 8);

#[cfg(test)]
mod test {

    #[test]
    fn test_ser() {
        use super::Serializable;
        
        // == test struct with no fields ==

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        struct SerTestNoField;

        let s1 = SerTestNoField;
        let mut buffer = [0u8; SerTestNoField::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestNoField::deserialize(&buffer);
        assert!(s1 == s2);

        // == test structs with unnamed fields ==

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        struct SerTestOneField(f32);

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        struct SerTestMultiField(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

        let s1 = SerTestOneField(3.1415);
        let mut buffer = [0u8; SerTestOneField::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestOneField::deserialize(&buffer);
        assert!(s1 == s2);

        let s1 = SerTestMultiField(1, 5, 354, 768, -3, -5480, 3456, -15338, 3.14, -764.89);
        let mut buffer = [0u8; SerTestMultiField::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestMultiField::deserialize(&buffer);
        assert!(s1 == s2);

        // == test structs with named fields

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        struct SerTestOneNamed {
            foo: f32,
        }

        let s1 = SerTestOneNamed { foo: 0.0234 };
        let mut buffer = [0u8; SerTestOneNamed::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestOneNamed::deserialize(&buffer);
        assert!(s1 == s2);

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        struct SerTestMultiNamed {
            foo: f32,
            bar: u8,
            baz: i64,
            qux: i8,
        }

        let s1 = SerTestMultiNamed { foo: 0.0234, bar: 4, baz: -34762672, qux: -43 };
        let mut buffer = [0u8; SerTestMultiNamed::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestMultiNamed::deserialize(&buffer);
        assert!(s1 == s2);

        // == test empty enum ==

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        enum SerTestEmptyEnum {
            Foo,
            Bar,
            Baz,
        }

        let s1 = SerTestEmptyEnum::Foo;
        let mut buffer = [0u8; SerTestEmptyEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestEmptyEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestEmptyEnum::Bar;
        let mut buffer = [0u8; SerTestEmptyEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestEmptyEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestEmptyEnum::Baz;
        let mut buffer = [0u8; SerTestEmptyEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestEmptyEnum::deserialize(&buffer);
        assert!(s1 == s2);

        // == test enum with unnamed fields == 

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        enum SerTestUnnamedEnum {
            Foo(u32, f64),
            Bar(i8),
            Baz(i64, i64, i64),
        }

        let s1 = SerTestUnnamedEnum::Foo(835473, 3.14157628);
        let mut buffer = [0u8; SerTestUnnamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestUnnamedEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestUnnamedEnum::Bar(-128);
        let mut buffer = [0u8; SerTestUnnamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestUnnamedEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestUnnamedEnum::Baz(-87623729867323, 618351835193651573, -12871271872812);
        let mut buffer = [0u8; SerTestUnnamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestUnnamedEnum::deserialize(&buffer);
        assert!(s1 == s2);

        // == test with enums and named fields

        #[derive(overcast_macros::Serializable)]
        #[derive(Clone, PartialEq)]
        enum SerTestNamedEnum {
            Foo{
                f1: u32,
                f2: f64
            },
            Bar {
                f1: i8
            },
            Baz{
                f1: i64,
                f2: i64,
                f3: i64,
            },
        }

        let s1 = SerTestNamedEnum::Foo{
            f1: 76429862,
            f2: 3.17861293,
        };
        let mut buffer = [0u8; SerTestNamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestNamedEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestNamedEnum::Bar {
            f1: -73
        };
        let mut buffer = [0u8; SerTestNamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestNamedEnum::deserialize(&buffer);
        assert!(s1 == s2);
        let s1 = SerTestNamedEnum::Baz{
            f1: -87623729867323,
            f2: 618351835193651573,
            f3: -12871271872812
        };
        let mut buffer = [0u8; SerTestNamedEnum::MAX_BIN_SIZE];
        s1.serialize(&mut buffer);
        let s2 = SerTestNamedEnum::deserialize(&buffer);
        assert!(s1 == s2);
    }
}
use std::ops::{
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Not,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
};

pub trait BitVector:
    Copy
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + ShlAssign<u32>
    + ShrAssign<u32>
{
    const BIT_COUNT: u32;
    const ZERO: Self;
}

impl BitVector for u8 {
    const BIT_COUNT: u32 = Self::BITS;
    const ZERO: Self = 0;
}

impl BitVector for u16 {
    const BIT_COUNT: u32 = Self::BITS;
    const ZERO: Self = 0;
}

impl BitVector for u32 {
    const BIT_COUNT: u32 = Self::BITS;
    const ZERO: Self = 0;
}

impl BitVector for u64 {
    const BIT_COUNT: u32 = Self::BITS;
    const ZERO: Self = 0;
}

impl BitVector for u128 {
    const BIT_COUNT: u32 = Self::BITS;
    const ZERO: Self = 0;
}

pub trait BitPack: Copy {
    type BitVector: BitVector;

    const BIT_COUNT: u32;

    fn pack(self) -> Self::BitVector;

    fn unpack(bits: Self::BitVector) -> Option<Self>;
}

pub fn write_packed<T>(buf: &mut [T::BitVector], index: usize, data: T)
where
    T: BitPack,
{
    let per_elem = <T::BitVector as BitVector>::BIT_COUNT / T::BIT_COUNT;
    let inter_elem = index / (per_elem as usize);
    let intra_elem = (index % (per_elem as usize)) as u32;

    let mut packed = data.pack();
    packed <<= intra_elem * T::BIT_COUNT;
    let mut mask = !<T::BitVector as BitVector>::ZERO;
    mask >>= <T::BitVector as BitVector>::BIT_COUNT - T::BIT_COUNT;
    mask <<= intra_elem * T::BIT_COUNT;
    let mut stored = buf[inter_elem];
    stored &= !mask;
    stored |= packed;
    buf[inter_elem] = stored;
}

pub fn read_packed<T>(
    buf: &[T::BitVector],
    index: usize,
) -> Result<T, T::BitVector>
where
    T: BitPack,
{
    let per_elem = <T::BitVector as BitVector>::BIT_COUNT / T::BIT_COUNT;
    let inter_elem = index / (per_elem as usize);
    let intra_elem = (index % (per_elem as usize)) as u32;

    let mut mask = !<T::BitVector as BitVector>::ZERO;
    mask >>= <T::BitVector as BitVector>::BIT_COUNT - T::BIT_COUNT;
    let mut packed = buf[inter_elem];
    packed >>= intra_elem * T::BIT_COUNT;
    packed &= mask;
    T::unpack(packed).ok_or(packed)
}

#[cfg(test)]
mod test {
    use crate::bitpack::read_packed;

    use super::{write_packed, BitPack};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u16)]
    pub enum Data {
        Foo = 0,
        Bar = 1,
        Baz = 2,
        Y = 3,
        X = 4,
    }

    impl Data {
        pub const FOO: u16 = Self::Foo as u16;
        pub const BAR: u16 = Self::Bar as u16;
        pub const BAZ: u16 = Self::Baz as u16;
        pub const Y_: u16 = Self::Y as u16;
        pub const X_: u16 = Self::X as u16;
    }

    impl BitPack for Data {
        type BitVector = u16;
        const BIT_COUNT: u32 = 3;

        fn pack(self) -> Self::BitVector {
            self as u16
        }

        fn unpack(bits: Self::BitVector) -> Option<Self> {
            Some(match bits {
                Self::FOO => Self::Foo,
                Self::BAR => Self::Bar,
                Self::BAZ => Self::Baz,
                Self::Y_ => Self::Y,
                Self::X_ => Self::X,
                _ => return None,
            })
        }
    }

    #[test]
    fn read_zeroed() {
        let buf = [0_u16; 4];
        for i in 0 .. 5 * 4 {
            assert_eq!(read_packed(&buf, i), Ok(Data::Foo));
        }
    }

    #[test]
    fn read_write() {
        let mut buf = [0_u16; 2];
        write_packed(&mut buf, 0, Data::X);
        write_packed(&mut buf, 1, Data::Y);
        write_packed(&mut buf, 2, Data::Bar);
        write_packed(&mut buf, 3, Data::Baz);
        write_packed(&mut buf, 4, Data::Baz);
        write_packed(&mut buf, 5, Data::Bar);
        write_packed(&mut buf, 6, Data::Y);
        write_packed(&mut buf, 7, Data::X);
        write_packed(&mut buf, 8, Data::X);
        write_packed(&mut buf, 9, Data::Baz);

        assert_eq!(read_packed(&buf, 0), Ok(Data::X));
        assert_eq!(read_packed(&buf, 1), Ok(Data::Y));
        assert_eq!(read_packed(&buf, 2), Ok(Data::Bar));
        assert_eq!(read_packed(&buf, 3), Ok(Data::Baz));
        assert_eq!(read_packed(&buf, 4), Ok(Data::Baz));
        assert_eq!(read_packed(&buf, 5), Ok(Data::Bar));
        assert_eq!(read_packed(&buf, 6), Ok(Data::Y));
        assert_eq!(read_packed(&buf, 7), Ok(Data::X));
        assert_eq!(read_packed(&buf, 8), Ok(Data::X));
        assert_eq!(read_packed(&buf, 9), Ok(Data::Baz));
    }

    #[test]
    fn overwrite() {
        let mut buf = [0_u16; 2];
        write_packed(&mut buf, 7, Data::X);
        write_packed(&mut buf, 7, Data::Foo);
        assert_eq!(read_packed(&buf, 7), Ok(Data::Foo));
    }
}

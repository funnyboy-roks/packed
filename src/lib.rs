//! # Packed
//!
//! A library for tightly packing and unpacking data in structs.
//!
//! We offer a single trait, `Packed`.  The type is implemented by default
//! on all integer types, booleans, arrays, and tuples.
//!
//! ## Usage
//!
//! ```rust
//! # use packed::Packed;
//! let my_number: u16 = 42;
//! let mut buf = [0u8; 3];
//! my_number.pack(&mut buf, 3);
//! assert_eq!(buf, [0b0000_0000, 0b0000_0101, 0b0100_0000]);
//! ```

#[cfg(test)]
mod tests;

pub trait Packed {
    /// Amount of bits that the packed struct takes up
    const SIZE: usize;
    // bytes: bytes from which to unpack this value
    // offset: bit offset from the first bit in the slice
    // TODO: how to force `bytes.len() * 8 - offset >= Self::SIZE`?
    fn unpack(bytes: &[u8], offset: usize) -> Self;
    // TODO: how to force `bytes.len() * 8 - offset >= Self::SIZE`?
    // offset: offset into the first byte: offset < 8
    fn pack(self, bytes: &mut [u8], offset: usize);

    // TODO: This should be const, but it can't because fuck you
    fn size_of_val(&self) -> usize {
        Self::SIZE
    }
}

impl Packed for bool {
    const SIZE: usize = 1;

    fn unpack(bytes: &[u8], offset: usize) -> Self {
        assert!(bytes.len() * 8 - offset >= Self::SIZE);
        let i = offset / 8;
        bytes[i] & 1 << (7 - offset % 8) != 0
    }

    fn pack(self, bytes: &mut [u8], offset: usize) {
        assert!(bytes.len() * 8 - offset >= Self::SIZE);
        bytes[offset / 8] &= !(1 << (7 - offset % 8));
        bytes[offset / 8] |= u8::from(self) << (7 - offset % 8);
    }
}

impl<T, const N: usize> Packed for [T; N]
where
    T: Packed,
{
    const SIZE: usize = N * T::SIZE;

    fn unpack(bytes: &[u8], offset: usize) -> Self {
        assert!(bytes.len() * 8 - offset >= Self::SIZE);
        std::array::from_fn::<_, N, _>(|i| T::unpack(bytes, offset + i * T::SIZE))
    }

    fn pack(self, bytes: &mut [u8], offset: usize) {
        assert!(bytes.len() * 8 - offset >= Self::SIZE);
        for (i, x) in self.into_iter().enumerate() {
            x.pack(bytes, offset + i * T::SIZE);
        }
    }
}

macro_rules! packed_int {
    ($ty: ident) => {
        impl Packed for $ty {
            const SIZE: usize = Self::BITS as _;

            fn unpack(bytes: &[u8], offset: usize) -> Self {
                assert!(bytes.len() * 8 - offset >= Self::SIZE);
                let bytes = &bytes[offset / 8..];
                let offset = offset % 8;

                let mut out = (bytes[0] as Self) << (offset);

                for b in bytes.iter().take(Self::BITS as usize / 8).skip(1) {
                    #[allow(arithmetic_overflow)]
                    {
                        out <<= 8;
                    }
                    out |= (*b as Self) << offset;
                }

                if offset != 0 {
                    out |= (bytes[std::mem::size_of::<Self>()] as Self) >> (8 - offset)
                }

                out
            }

            fn pack(self, bytes: &mut [u8], offset: usize) {
                assert!(bytes.len() * 8 - offset >= Self::SIZE);
                let bytes = &mut bytes[offset / 8..];
                let offset = offset % 8;
                let bytes01 = (self >> offset).to_be_bytes();

                if offset != 0 {
                    bytes[0] &= !((1 << (8 - offset)) - 1);
                    bytes[0] |= bytes01[0];
                } else {
                    bytes[0] = bytes01[0];
                }

                for (i, b) in bytes01.into_iter().enumerate().skip(1) {
                    bytes[i] = b;
                }

                if offset != 0 {
                    let mask: u8 = (1 << offset) - 1;
                    let byte = &mut bytes[std::mem::size_of::<Self>()];
                    let b = *byte & !(mask << (8 - offset));
                    *byte = (self as u8 & mask) << (8 - offset) | b;
                }
            }
        }
    };
    // this is kind of cursed, but it's finnneeeee
    ($from: ident => $to: ident) => {
        impl Packed for $to {
            const SIZE: usize = <$from>::SIZE;

            fn unpack(bytes: &[u8], offset: usize) -> Self {
                $from::unpack(bytes, offset) as _
            }

            fn pack(self, bytes: &mut [u8], offset: usize) {
                (self as $from).pack(bytes, offset);
            }
        }
    };
    ($($ty: ident),+$(,)?) => {
        $(packed_int!($ty);)+
    };
    ($($from: ident => $to: ident),+$(,)?) => {
        $(packed_int!($from => $to);)+
    };
}

packed_int!(u8, u16, u32, u64, u128, usize);
packed_int!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128, usize => isize);

impl Packed for () {
    const SIZE: usize = 0;

    fn unpack(_: &[u8], _: usize) -> Self {}

    fn pack(self, _: &mut [u8], _: usize) {}
}

macro_rules! tuple_impl {
    (()) => {};
    (($($x: ident,)+)) => {
        tuple_impl!(@tail2 $($x,)+);

        impl<$($x,)+> Packed for ($($x,)+)
        where
            $($x: Packed,)+
        {
            const SIZE: usize = <tuple_impl!(@head $($x,)+)>::SIZE + <tuple_impl!(@tail $($x,)+)>::SIZE;

            #[allow(unused_assignments)]
            fn unpack(bytes: &[u8], mut offset: usize) -> Self {
                (
                    $({
                        let x = $x::unpack(bytes, offset);
                        offset += $x::SIZE;
                        x
                    },)+
                )
            }

            fn pack(self, bytes: &mut [u8], offset: usize) {
                let a_sz = <tuple_impl!(@head $($x,)+)>::SIZE;
                #[allow(non_snake_case)]
                let ($($x,)+) = self;
                tuple_impl!(@head $($x,)+).pack(bytes, offset);
                tuple_impl!(@tail $($x,)+).pack(bytes, offset + a_sz);
            }
        }
    };
    (@head $x: ident, $($_: ident,)*) => {
        $x
    };
    (@tail $_: ident, $($x: ident,)*) => {
        ($($x,)*)
    };
    (@tail2 $_: ident, $($x: ident,)*) => {
        tuple_impl!(($($x,)*));
    };
}

tuple_impl!((A, B, C, D, E, F, G, H, I, J, K, L,));

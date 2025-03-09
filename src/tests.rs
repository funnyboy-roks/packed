use crate::Packed;

fn unpack_from_val<T>(_: &T, bytes: &[u8], offset: usize) -> T
where
    T: Packed,
{
    T::unpack(bytes, offset)
}

macro_rules! bin_dbg {
    ($slice: expr) => {
        std::eprint!(
            "[{}:{}:{}] {} = ",
            std::file!(),
            std::line!(),
            std::column!(),
            std::stringify!($slice)
        );
        for x in $slice {
            eprint!("{:08b} ", x);
        }
        eprintln!();
    };
}

macro_rules! simple {
    ($name: ident ($type: ty)) => {
        #[test]
        fn $name() {
            proptest::proptest!(|(n: $type)| {
                let mut buf = [0u8; <$type>::SIZE / 8 + 3];
                for offset in 0..=16 {
                    n.pack(&mut buf, offset);
                    let m = <$type>::unpack(&buf, offset);
                    bin_dbg!(buf);
                    assert_eq!(n, m);
                }
            });
        }
    };
}

simple!(pack_unpack_bools([bool; 14]));
simple!(pack_unpack_bools_long([bool; 64]));
simple!(pack_unpack_u8(u8));
simple!(pack_unpack_u16(u16));
simple!(pack_unpack_u32(u32));
simple!(pack_unpack_u64(u64));
simple!(pack_unpack_u128(u128));
simple!(pack_unpack_usize(usize));
simple!(pack_unpack_i8(i8));
simple!(pack_unpack_i16(i16));
simple!(pack_unpack_i32(i32));
simple!(pack_unpack_i64(i64));
simple!(pack_unpack_i128(i128));
simple!(pack_unpack_isize(isize));
simple!(pack_unpack_int_tuple1((u8, u16, u32)));
simple!(pack_unpack_int_tuple2((i8, u16, i128)));
simple!(pack_unpack_int_tuple3((u8, u16, u32, u64, u128, usize)));
simple!(pack_unpack_int_tuple4((i8, i16, i32, i64, i128, isize)));

proptest::proptest! {
    #[test]
    fn pack_unpack_tuple(tuple: (u16, bool, u16, bool)) {
        assert_eq!(tuple.size_of_val(), 34);
        let mut manual = [0u8; 8];
        let mut packed = [0u8; 8];
        for offset in 0..=16 {
            tuple.0.pack(&mut manual, offset);
            tuple.1.pack(&mut manual, offset + tuple.0.size_of_val());
            tuple.2.pack(
                &mut manual,
                offset + tuple.0.size_of_val() + tuple.1.size_of_val(),
            );
            tuple.3.pack(
                &mut manual,
                offset + tuple.0.size_of_val() + tuple.1.size_of_val() + tuple.2.size_of_val(),
            );

            tuple.pack(&mut packed, offset);
            bin_dbg!(manual);
            bin_dbg!(packed);
            assert_eq!(manual, packed);

            let unpacked = unpack_from_val(&tuple, &packed, offset);
            assert_eq!(tuple, unpacked);
        }
    }
}

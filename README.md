# Packed

A library for tightly packing and unpacking data in structs

We offer a single trait, `Packed`.  The type is implemented by default
on all integer types, booleans, arrays, and tuples.

## Usage

```rust
let my_number: u16 = 42;
let mut buf = [0u8; 3];
my_number.pack(&mut buf, 3);
assert_eq!(buf, [0b0000_0000, 0b0000_0101, 0b0100_0000]);
```

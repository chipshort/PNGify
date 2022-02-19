pub const U64_BYTES: usize = (u64::BITS / u8::BITS) as usize;
const U64_BYTES_U32: u32 = (u64::BITS / u8::BITS) as u32;

pub trait Bytes {
    fn write_u64(&mut self, pos: usize, value: u64);
    fn read_u64(&self, pos: usize) -> u64;
}

impl Bytes for [u8] {
    /// Writes the given value at the given position (in reversed byte order)
    /// ```
    /// let vec = vec![0x07u8, 0x06u8, 0x05u8, 0x04u8, 0x03u8, 0x02u8, 0x01u8, 0x00u8];
    /// assert_eq!(vec.read_u64(0), 0x0001020304050607u64);
    /// ```
    fn write_u64(&mut self, pos: usize, value: u64) {
        assert!(self.len() >= pos + U64_BYTES);
        for i in 0..U64_BYTES_U32 {
            self[pos + i as usize] = ((value >> (i * u8::BITS)) & 0xFFu64) as u8;
        }
    }

    /// Reads the u64 (in reversed byte order) at the given position
    /// ```
    /// let mut vec = vec![0; U64_BYTES];
    /// vec.write_u64(0, 0x00FF00FF00FF00FFu64);
    /// assert_eq!(&vec, &[0xFFu8, 0x00u8, 0xFFu8, 0x00u8, 0xFFu8, 0x00u8, 0xFFu8, 0x00u8]);
    /// ```
    fn read_u64(&self, pos: usize) -> u64 {
        assert!(self.len() >= pos + U64_BYTES);
        let mut result = 0u64;
        for i in 0..U64_BYTES_U32 {
            result |= (self[pos + i as usize] as u64) << (i * u8::BITS);
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{Bytes, U64_BYTES};

    #[test]
    fn test_read_bytes() {
        let vec = vec![
            0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
        ];
        assert_eq!(vec.read_u64(0), u64::MAX);

        let vec = vec![
            0x07u8, 0x06u8, 0x05u8, 0x04u8, 0x03u8, 0x02u8, 0x01u8, 0x00u8,
        ];
        assert_eq!(vec.read_u64(0), 0x0001020304050607u64);
    }

    #[test]
    fn test_write_bytes() {
        let mut vec = vec![0; U64_BYTES];
        vec.write_u64(0, u64::MAX);
        assert_eq!(&vec, &[0xFFu8; U64_BYTES]);

        vec.write_u64(0, 0x0001020304050607u64);
        assert_eq!(
            &vec,
            &[0x07u8, 0x06u8, 0x05u8, 0x04u8, 0x03u8, 0x02u8, 0x01u8, 0x00u8]
        );
    }
}

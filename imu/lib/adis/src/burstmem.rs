use duplicate;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

type Word = u16;

pub trait BurstMemory {
    fn diag_stat(&self) -> u16;
    fn xa(&self) -> f64;
    fn ya(&self) -> f64;
    fn za(&self) -> f64;
    fn xb(&self) -> f64;
    fn yb(&self) -> f64;
    fn zb(&self) -> f64;
    fn temp(&self) -> f64;
    fn data_cntr(&self) -> u16;
    fn is_corrupted(&self) -> bool;
}

// all sizes are in double bytes (16 bits)
mod size {
    const WORD_SIZE_BITS: usize = super::Word::BITS as usize;

    pub const DIAG_STAT: usize = 1;
    pub const DATA_FIELD: usize = 1;
    pub const TEMP: usize = 1;
    pub const DATA_CNTR: usize = 1;
    pub const CHECKSUM: usize = 1;

    const fn data_field_size(data_size: usize) -> usize {
        return (data_size / WORD_SIZE_BITS) * DATA_FIELD * 6;
    }

    pub const fn burst_size(data_size: usize) -> usize {
        return DIAG_STAT + data_field_size(data_size) + TEMP + DATA_CNTR + CHECKSUM;
    }

    pub const fn diag_stat_off(_data_size: usize) -> usize {
        return 0;
    }

    pub const fn data_field_off(_data_size: usize) -> usize {
        return DIAG_STAT;
    }

    pub const fn temp_off(data_size: usize) -> usize {
        return DIAG_STAT + data_field_size(data_size);
    }

    pub const fn data_cntr_off(data_size: usize) -> usize {
        return DIAG_STAT + data_field_size(data_size) + TEMP;
    }

    pub const fn checksum_off(data_size: usize) -> usize {
        return DIAG_STAT + data_field_size(data_size) + TEMP + DATA_CNTR;
    }
}

duplicate::duplicate! {
    [
        struct_name     data_size;
        [BurstMemory16] [16];
        [BurstMemory32] [32];
    ]

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct struct_name {
        data: [Word; size::burst_size(data_size)],
    }

    impl Default for struct_name {
        fn default() -> Self {
            return Self {
                data: [0; size::burst_size(data_size)],
            };
        }
    }

    impl From<[Word; size::burst_size(data_size)]> for struct_name {
        fn from(data: [Word; size::burst_size(data_size)]) -> Self {
            return Self {
                data,
            };
        }
    }

    impl BurstMemory for struct_name {
        #[inline(always)]
        fn diag_stat(&self) -> u16 {
            return *word_transmute_u16(&self.data[size::diag_stat_off(data_size)]);
        }

        #[inline(always)]
        fn temp(&self) -> f64 {
            return *word_transmute_i16(&self.data[size::temp_off(data_size)]) as f64;
        }

        #[inline(always)]
        fn data_cntr(&self) -> u16 {
            return *word_transmute_u16(&self.data[size::data_cntr_off(data_size)]);
        }


        fn is_corrupted(&self) -> bool {

            let data: &[u8; size::burst_size(data_size) * 2 ] = unsafe { core::mem::transmute(&self.data) };
            let mut checksum: u16 = 0;

            for i in 0..(size::burst_size(data_size)*2) - 2 {
                checksum = checksum.wrapping_add(data[i] as u16);
            }
            return checksum != self.checksum();
        }

        #[duplicate::duplicate_item(
            fnc_name    index;
            [xa]        [0];
            [ya]        [1];
            [za]        [2];
            [xb]        [3];
            [yb]        [4];
            [zb]        [5];
        )]
        #[inline(always)]
        fn fnc_name(&self) -> f64 {
            return if data_size == 16 {
                *self.data_field(index) as f64
            } else {
                *self.data_field(2* index) as f64 + (*self.data_field(2* index + 1) as f64 / (1<<15) as f64)
            };
        }
    }

    impl struct_name {
        #[inline(always)]
        pub fn checksum(&self) -> u16 {
            return *word_transmute_u16(&self.data[size::checksum_off(data_size)]);
        }
    }


    impl<'a> struct_name {
        #[inline(always)]
        fn data_field(&'a self, index: usize) -> &'a i16 {
            return word_transmute_i16(&self.data[size::data_field_off(data_size) + index]);
        }
    }

}

#[inline]
fn word_transmute_u16<'a>(x: &'a Word) -> &'a u16 {
    return unsafe { core::mem::transmute(x) };
}

#[inline]
fn word_transmute_i16<'a>(x: &'a Word) -> &'a i16 {
    return unsafe { core::mem::transmute(x) };
}

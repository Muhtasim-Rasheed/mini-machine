#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct RadixByte<const BASE: u16, const DIGITS: usize>(u16);

impl<const BASE: u16, const DIGITS: usize> RadixByte<BASE, DIGITS> {
    // u32 avoids overflow while computing BASE.pow(DIGITS) for larger bases/digit-counts,
    // even though the final stored value always fits in u16 for reasonable esolang sizes.
    pub const MODULUS: u32 = (BASE as u32).pow(DIGITS as u32);

    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self((Self::MODULUS - 1) as u16);

    pub const fn new(raw: u16) -> Self {
        // debug_assert instead of a hard panic keeps const contexts usable in release builds
        debug_assert!((raw as u32) < Self::MODULUS);
        Self(raw)
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn digits(self) -> [u8; DIGITS] {
        let mut out = [0u8; DIGITS];
        let mut val = self.0;
        let mut i = DIGITS;
        while i > 0 {
            i -= 1;
            out[i] = (val % BASE) as u8;
            val /= BASE;
        }
        out
    }

    pub const fn from_digits(digits: [u8; DIGITS]) -> Self {
        let mut val: u32 = 0;
        let mut i = 0;
        while i < DIGITS {
            val = val * BASE as u32 + digits[i] as u32;
            i += 1;
        }
        Self(val as u16)
    }

    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self(((self.0 as u32 + rhs.0 as u32) % Self::MODULUS) as u16)
    }

    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self(((Self::MODULUS + self.0 as u32 - rhs.0 as u32) % Self::MODULUS) as u16)
    }

    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        // u16 * u16 fits comfortably in u32, no overflow risk even at base-17-sized values
        Self(((self.0 as u32 * rhs.0 as u32) % Self::MODULUS) as u16)
    }

    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        Some(Self((self.0 / rhs.0) % Self::MODULUS as u16))
    }

    pub const fn checked_mod(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        Some(Self(self.0 % rhs.0))
    }

    pub const fn wrapping_neg(self) -> Self {
        Self(((Self::MODULUS - self.0 as u32) % Self::MODULUS) as u16)
    }

    pub const fn dnot(self) -> Self {
        let mut digits = self.digits();
        let mut i = 0;
        while i < DIGITS as usize {
            digits[i] = (Self::MODULUS - 1 - digits[i] as u32) as u8;
            i += 1;
        }
        Self::from_digits(digits)
    }

    pub const fn dmin(self, rhs: Self) -> Self {
        let mut digits1 = self.digits();
        let digits2 = rhs.digits();
        let mut i = 0;
        while i < DIGITS as usize {
            digits1[i] = if digits1[i] < digits2[i] {
                digits1[i]
            } else {
                digits2[i]
            };
            i += 1;
        }
        Self::from_digits(digits1)
    }

    pub const fn dmax(self, rhs: Self) -> Self {
        let mut digits1 = self.digits();
        let digits2 = rhs.digits();
        let mut i = 0;
        while i < DIGITS as usize {
            digits1[i] = if digits1[i] > digits2[i] {
                digits1[i]
            } else {
                digits2[i]
            };
            i += 1;
        }
        Self::from_digits(digits1)
    }

    pub const fn as_signed(self) -> i16 {
        if self.0 <= Self::MAX.0 / 2 {
            self.0 as i16
        } else {
            self.0 as i16 - Self::MODULUS as i16
        }
    }
}

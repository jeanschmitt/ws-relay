use std::array::TryFromSliceError;
use std::fmt::{Display, Formatter};

pub const CODE_SIZE: usize = 4;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Code([u8; CODE_SIZE]);

impl Code {
    pub fn new() -> Self {
        Code(rand::random())
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<&[u8]> for Code {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Code(value.try_into()?))
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a Code {
    type Error = TryFromSliceError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let inner: &'a [u8; 4] = value.try_into()?;

        // SAFETY: Code is #[repr(transparent)], and value.len() was checked with value.try_into()
        let code = unsafe { &*(inner as *const [u8; 4] as *const Code) };

        Ok(code)
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ref() {
        let buf: &[u8; 4] = &[1, 2, 3, 4];

        let code: &Code = buf[0..4].try_into().unwrap();

        assert_eq!([1, 2, 3, 4], code.0);
    }
}

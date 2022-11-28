pub const CODE_SIZE: usize = 4;

pub type Code = [u8; CODE_SIZE];

pub fn new_code() -> Code {
    rand::random()
}

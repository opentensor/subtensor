use subtensor_macros::freeze_struct;

#[freeze_struct("18ec48d6e1ccaa1b")]
pub struct MyStruct {
    pub ip: u32,
    pub port: u32,
}

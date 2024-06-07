use subtensor_macros::freeze_struct;

#[freeze_struct("1f29b57a32dbb617")]
struct MyStruct {
    ip: u32,
    port: u32,
}

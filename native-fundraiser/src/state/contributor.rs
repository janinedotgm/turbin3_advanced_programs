use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Contributor {
    pub amount: u64,
}  

impl Contributor {
    pub const LEN: usize = std::mem::size_of::<Contributor>();
}
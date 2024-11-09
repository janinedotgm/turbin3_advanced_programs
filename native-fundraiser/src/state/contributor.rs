use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Contributor {
    pub amount: u64,
    pub contributor_bump: u64,
}  

// TODO: How to get rid of this warning?
impl Contributor {
    pub const LEN: usize = std::mem::size_of::<Contributor>();
}
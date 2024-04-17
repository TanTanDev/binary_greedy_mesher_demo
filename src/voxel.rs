#[repr(u32)]
#[derive(Eq, PartialEq, Default, Copy, Clone, Debug)]
pub enum BlockType {
    #[default]
    Air,
    Grass,
    Dirt,
}

pub const MESHABLE_BLOCK_TYPES: &'static [BlockType] = &[BlockType::Grass, BlockType::Dirt];

impl BlockType {
    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Air => false,
            BlockType::Grass => true,
            BlockType::Dirt => true,
        }
    }
    pub fn is_air(&self) -> bool {
        !self.is_solid()
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct BlockData {
    pub block_type: BlockType,
}

///! level of detail
#[derive(Copy, Clone)]
pub enum Lod {
    L32,
    L16,
    L8,
    L4,
    L2,
}

impl Lod {
    ///! the amount of voxels per axis
    pub fn size(&self) -> i32 {
        match self {
            Lod::L32 => 32,
            Lod::L16 => 16,
            Lod::L8 => 8,
            Lod::L4 => 4,
            Lod::L2 => 2,
        }
    }

    ///! how much to multiply to reach next voxel
    ///! lower lod gives higher jump
    pub fn jump_index(&self) -> i32 {
        match self {
            Lod::L32 => 1,
            Lod::L16 => 2,
            Lod::L8 => 4,
            Lod::L4 => 8,
            Lod::L2 => 16,
        }
    }
}

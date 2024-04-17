use crate::lod::Lod;
use bevy::math::{ivec3, IVec3};

// helper for transforming translations based dir or "axis"
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FaceDir {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Back,
}

impl FaceDir {
    ///! normal data is packed in the shader
    pub fn normal_index(&self) -> u32 {
        match self {
            FaceDir::Left => 0u32,
            FaceDir::Right => 1u32,
            FaceDir::Down => 2u32,
            FaceDir::Up => 3u32,
            FaceDir::Forward => 4u32,
            FaceDir::Back => 5u32,
        }
    }

    ///! direction to sample face culling
    pub fn air_sample_dir(&self) -> IVec3 {
        match self {
            FaceDir::Up => IVec3::Y,
            FaceDir::Down => IVec3::NEG_Y,
            FaceDir::Left => IVec3::NEG_X,
            FaceDir::Right => IVec3::X,
            FaceDir::Forward => IVec3::NEG_Z,
            FaceDir::Back => IVec3::Z,
        }
    }

    ///! offset input position with this face direction
    pub fn world_to_sample(&self, axis: i32, x: i32, y: i32, _lod: &Lod) -> IVec3 {
        match self {
            FaceDir::Up => ivec3(x, axis + 1, y),
            FaceDir::Down => ivec3(x, axis, y),
            FaceDir::Left => ivec3(axis, y, x),
            FaceDir::Right => ivec3(axis + 1, y, x),
            FaceDir::Forward => ivec3(x, y, axis),
            FaceDir::Back => ivec3(x, y, axis + 1),
        }
    }

    ///! returns true if vertices should be reverse.
    ///! (needed because indices are always same)  
    pub fn reverse_order(&self) -> bool {
        match self {
            FaceDir::Up => true,      //+1
            FaceDir::Down => false,   //-1
            FaceDir::Left => false,   //-1
            FaceDir::Right => true,   //+1
            FaceDir::Forward => true, //-1
            FaceDir::Back => false,   //+1
        }
    }

    ///! get delta for traversing the previous axis pos
    pub fn negate_axis(&self) -> i32 {
        match self {
            FaceDir::Up => -1,     //+1
            FaceDir::Down => 0,    //-1
            FaceDir::Left => 0,    //-1
            FaceDir::Right => -1,  //+1
            FaceDir::Forward => 0, //-1
            FaceDir::Back => 1,    //+1
        }
    }
}

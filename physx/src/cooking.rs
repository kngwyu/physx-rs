// We've deprecated PxCooking...but this is the implementation :p
#![allow(deprecated)]

use crate::{
    bvh::Bvh, convex_mesh::ConvexMesh, foundation::Foundation, height_field::HeightField,
    owner::Owner, physics::Physics, traits::Class, triangle_mesh::TriangleMesh,
};

#[rustfmt::skip]
use physx_sys::{
    phys_PxCreateCooking,
    PxBVHDesc_isValid,
    PxBVHDesc_new,
    PxBVHDesc_setToDefault_mut,
    PxConvexMeshDesc_isValid,
    PxConvexMeshDesc_new,
    PxConvexMeshDesc_setToDefault_mut,
    PxCookingParams_new,
    PxCooking_createBVH,
    PxCooking_createConvexMesh,
    PxCooking_createHeightField,
    PxCooking_createTriangleMesh,
    PxCooking_release_mut,
    PxCooking_validateConvexMesh,
    PxCooking_validateTriangleMesh,
    PxHeightFieldDesc_isValid,
    PxHeightFieldDesc_new,
    PxHeightFieldDesc_setToDefault_mut,
    PxTriangleMeshDesc_isValid,
    PxTriangleMeshDesc_new,
    PxTriangleMeshDesc_setToDefault_mut,
};

/// A new-type wrapper around `physx_sys::PxCooking`.
#[deprecated(
    since = "0.15.0",
    note = "The PxCooking interface has been deprecated, but for the initial upgrade to PhysX 5 we still support it"
)]
pub struct PxCooking {
    obj: physx_sys::PxCooking,
}

unsafe impl Class<physx_sys::PxCooking> for PxCooking {
    fn as_ptr(&self) -> *const physx_sys::PxCooking {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxCooking {
        &mut self.obj
    }
}

impl Drop for PxCooking {
    fn drop(&mut self) {
        unsafe { PxCooking_release_mut(self.as_mut_ptr()) }
    }
}

unsafe impl Send for PxCooking {}
unsafe impl Sync for PxCooking {}

impl PxCooking {
    /// Create a new cooking instance.
    pub fn new(foundation: &mut impl Foundation, params: &PxCookingParams) -> Option<Owner<Self>> {
        unsafe {
            Owner::from_raw(phys_PxCreateCooking(
                crate::physics::PX_PHYSICS_VERSION,
                foundation.as_mut_ptr(),
                params.as_ptr(),
            ) as *mut _)
        }
    }

    /// Cook a new BVH
    pub fn create_bvh(&self, physics: &mut impl Physics, desc: &PxBVHDesc) -> Option<Owner<Bvh>> {
        unsafe {
            Bvh::from_raw(PxCooking_createBVH(
                self.as_ptr(),
                desc.as_ptr(),
                physics.get_physics_insertion_callback()?,
            ))
        }
    }

    /// Cook a new convex mesh using a mesh descriptor.
    /// The data provided in the mesh descriptor should be validated by [`PxCooking::validate_convex_mesh`] before being
    /// passed into this method.
    pub fn create_convex_mesh(
        &self,
        physics: &mut impl Physics,
        desc: &PxConvexMeshDesc,
    ) -> ConvexMeshCookingResult {
        if !desc.is_valid() {
            return ConvexMeshCookingResult::InvalidDescriptor;
        };
        if let Some(callback) = physics.get_physics_insertion_callback() {
            let mut result = ConvRes::Failure;
            let ptr = unsafe {
                ConvexMesh::from_raw(PxCooking_createConvexMesh(
                    self.as_ptr(),
                    desc.as_ptr(),
                    callback,
                    &mut result,
                ))
            };
            ConvexMeshCookingResult::from_raw(result, ptr)
        } else {
            ConvexMeshCookingResult::Failure
        }
    }

    /// Cook a new height field.
    pub fn create_height_field(
        &self,
        physics: &mut impl Physics,
        desc: &PxHeightFieldDesc,
    ) -> Option<Owner<HeightField>> {
        unsafe {
            HeightField::from_raw(PxCooking_createHeightField(
                self.as_ptr(),
                desc.as_ptr(),
                physics.get_physics_insertion_callback()?,
            ))
        }
    }

    /// Cook a new triangle mesh using a mesh descriptor.
    /// The data provided in the mesh descriptor should be validated by [`PxCooking::validate_triangle_mesh`] before being
    /// passed into this method.
    pub fn create_triangle_mesh(
        &self,
        physics: &mut impl Physics,
        desc: &PxTriangleMeshDesc,
    ) -> TriangleMeshCookingResult {
        if !desc.is_valid() {
            return TriangleMeshCookingResult::InvalidDescriptor;
        };
        if let Some(callback) = physics.get_physics_insertion_callback() {
            let mut result = TriResult::Failure;
            let ptr = unsafe {
                TriangleMesh::from_raw(PxCooking_createTriangleMesh(
                    self.as_ptr(),
                    desc.as_ptr(),
                    callback,
                    &mut result,
                ))
            };
            TriangleMeshCookingResult::from_raw(result, ptr)
        } else {
            TriangleMeshCookingResult::Failure
        }
    }

    /// Validate a convex mesh descriptor.
    pub fn validate_convex_mesh(&self, desc: &PxConvexMeshDesc) -> bool {
        unsafe { PxCooking_validateConvexMesh(self.as_ptr(), desc.as_ptr()) }
    }

    /// Validate a triangle mesh descriptor.
    pub fn validate_triangle_mesh(&self, desc: &PxTriangleMeshDesc) -> bool {
        unsafe { PxCooking_validateTriangleMesh(self.as_ptr(), desc.as_ptr()) }
    }
}

use physx_sys::PxConvexMeshCookingResult as ConvRes;

pub enum ConvexMeshCookingResult {
    Success(Owner<ConvexMesh>),
    ZeroAreaTestFailed,
    PolygonsLimitReached,
    Failure,
    InvalidDescriptor,
}

impl ConvexMeshCookingResult {
    fn from_raw(px_result: ConvRes, ptr: Option<Owner<ConvexMesh>>) -> Self {
        match px_result {
            ConvRes::Success => {
                if let Some(ptr) = ptr {
                    Self::Success(ptr)
                } else {
                    Self::Failure
                }
            }
            ConvRes::ZeroAreaTestFailed => Self::ZeroAreaTestFailed,
            ConvRes::PolygonsLimitReached => Self::PolygonsLimitReached,
            ConvRes::Failure => Self::Failure,
        }
    }
}

use physx_sys::PxTriangleMeshCookingResult as TriResult;

pub enum TriangleMeshCookingResult {
    Success(Owner<TriangleMesh>),
    LargeTriangle,
    Failure,
    InvalidDescriptor,
}

impl TriangleMeshCookingResult {
    fn from_raw(px_result: TriResult, ptr: Option<Owner<TriangleMesh>>) -> Self {
        match px_result {
            TriResult::Success => {
                if let Some(ptr) = ptr {
                    Self::Success(ptr)
                } else {
                    Self::Failure
                }
            }
            TriResult::LargeTriangle => Self::LargeTriangle,
            TriResult::Failure => Self::Failure,
        }
    }
}

pub struct PxConvexMeshDesc {
    pub obj: physx_sys::PxConvexMeshDesc,
}

unsafe impl Class<physx_sys::PxConvexMeshDesc> for PxConvexMeshDesc {
    fn as_ptr(&self) -> *const physx_sys::PxConvexMeshDesc {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxConvexMeshDesc {
        &mut self.obj
    }
}

impl Default for PxConvexMeshDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl PxConvexMeshDesc {
    /// Create a new convex mesh descriptor.
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: PxConvexMeshDesc_new(),
            }
        }
    }

    /// Check if the descriptor is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { PxConvexMeshDesc_isValid(self.as_ptr()) }
    }

    /// Set the descriptor to its default values.
    pub fn set_to_default(&mut self) -> &mut Self {
        unsafe {
            PxConvexMeshDesc_setToDefault_mut(self.as_mut_ptr());
        }
        self
    }
}

pub struct PxTriangleMeshDesc {
    pub obj: physx_sys::PxTriangleMeshDesc,
}

unsafe impl Class<physx_sys::PxTriangleMeshDesc> for PxTriangleMeshDesc {
    fn as_ptr(&self) -> *const physx_sys::PxTriangleMeshDesc {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxTriangleMeshDesc {
        &mut self.obj
    }
}

impl Default for PxTriangleMeshDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl PxTriangleMeshDesc {
    /// Create a new triangle mesh descriptor.
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: PxTriangleMeshDesc_new(),
            }
        }
    }

    /// Check if the descriptor is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { PxTriangleMeshDesc_isValid(self.as_ptr()) }
    }

    /// Set the descriptor to its default values.
    pub fn set_to_default(&mut self) -> &mut Self {
        unsafe {
            PxTriangleMeshDesc_setToDefault_mut(self.as_mut_ptr());
        }
        self
    }
}

pub struct PxHeightFieldDesc {
    pub obj: physx_sys::PxHeightFieldDesc,
}

unsafe impl Class<physx_sys::PxHeightFieldDesc> for PxHeightFieldDesc {
    fn as_ptr(&self) -> *const physx_sys::PxHeightFieldDesc {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxHeightFieldDesc {
        &mut self.obj
    }
}

impl Default for PxHeightFieldDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl PxHeightFieldDesc {
    /// Create a new height field descriptor.
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: PxHeightFieldDesc_new(),
            }
        }
    }

    /// Check if the descriptor is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { PxHeightFieldDesc_isValid(self.as_ptr()) }
    }

    /// Set the descriptor to its default values.
    pub fn set_to_default(&mut self) -> &mut Self {
        unsafe {
            PxHeightFieldDesc_setToDefault_mut(self.as_mut_ptr());
        }
        self
    }
}

pub struct PxBVHDesc {
    pub obj: physx_sys::PxBVHDesc,
}

unsafe impl Class<physx_sys::PxBVHDesc> for PxBVHDesc {
    fn as_ptr(&self) -> *const physx_sys::PxBVHDesc {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxBVHDesc {
        &mut self.obj
    }
}

impl Default for PxBVHDesc {
    fn default() -> Self {
        Self::new()
    }
}

impl PxBVHDesc {
    /// Create a new BVH structure descriptor.
    pub fn new() -> Self {
        unsafe {
            Self {
                obj: PxBVHDesc_new(),
            }
        }
    }

    /// Check if the descriptor is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { PxBVHDesc_isValid(self.as_ptr()) }
    }

    /// Set the descriptor to its default values.
    pub fn set_to_default(&mut self) -> &mut Self {
        unsafe {
            PxBVHDesc_setToDefault_mut(self.as_mut_ptr());
        }
        self
    }
}

pub struct PxCookingParams {
    pub obj: physx_sys::PxCookingParams,
}

unsafe impl Class<physx_sys::PxCookingParams> for PxCookingParams {
    fn as_ptr(&self) -> *const physx_sys::PxCookingParams {
        &self.obj
    }

    fn as_mut_ptr(&mut self) -> *mut physx_sys::PxCookingParams {
        &mut self.obj
    }
}

impl PxCookingParams {
    /// Create a new cooking params.
    pub fn new(physics: &impl Physics) -> Option<Self> {
        unsafe {
            physics.get_tolerances_scale().map(|tolerances| Self {
                obj: PxCookingParams_new(tolerances),
            })
        }
    }
}

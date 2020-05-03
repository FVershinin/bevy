use super::{BindGroupDescriptor, VertexBufferDescriptor, VertexBufferDescriptors};
use crate::shader::ShaderLayout;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug, Default)]
pub struct PipelineLayout {
    pub bind_groups: Vec<BindGroupDescriptor>,
    pub vertex_buffer_descriptors: Vec<VertexBufferDescriptor>,
}

impl PipelineLayout {
    pub fn from_shader_layouts(shader_layouts: &mut [ShaderLayout]) -> Self {
        let mut bind_groups = HashMap::<u32, BindGroupDescriptor>::new();
        let mut vertex_buffer_descriptors = Vec::new();
        for shader_layout in shader_layouts.iter_mut() {
            for shader_bind_group in shader_layout.bind_groups.iter_mut() {
                match bind_groups.get_mut(&shader_bind_group.index) {
                    Some(bind_group) => {
                        for shader_binding in shader_bind_group.bindings.iter() {
                            if let Some(binding) = bind_group
                                .bindings
                                .iter()
                                .find(|binding| binding.index == shader_binding.index)
                            {
                                if binding != shader_binding {
                                    panic!("Binding {} in BindGroup {} does not match across all shader types: {:?} {:?}", binding.index, bind_group.index, binding, shader_binding);
                                }
                            } else {
                                bind_group.bindings.push(shader_binding.clone());
                            }
                        }
                    }
                    None => {
                        bind_groups.insert(shader_bind_group.index, shader_bind_group.clone());
                    }
                }
            }
        }

        for vertex_buffer_descriptor in shader_layouts[0].vertex_buffer_descriptors.iter() {
            vertex_buffer_descriptors.push(vertex_buffer_descriptor.clone());
        }

        let mut bind_groups_result = bind_groups
            .drain()
            .map(|(_, value)| value)
            .collect::<Vec<BindGroupDescriptor>>();

        // NOTE: for some reason bind groups need to be sorted by index. this is likely an issue with bevy and not with wgpu
        // TODO: try removing this
        bind_groups_result.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

        PipelineLayout {
            bind_groups: bind_groups_result,
            vertex_buffer_descriptors,
        }
    }

    pub fn sync_vertex_buffer_descriptors(
        &mut self,
        vertex_buffer_descriptors: &VertexBufferDescriptors,
    ) {
        for vertex_buffer_descriptor in self.vertex_buffer_descriptors.iter_mut() {
            if let Some(graph_descriptor) =
                vertex_buffer_descriptors.get(&vertex_buffer_descriptor.name)
            {
                vertex_buffer_descriptor.sync_with_descriptor(graph_descriptor);
            } else {
                panic!(
                    "Encountered unsupported Vertex Buffer: {}",
                    vertex_buffer_descriptor.name
                );
            }
        }
    }
}

#[derive(Hash, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UniformProperty {
    pub name: String,
    pub property_type: UniformPropertyType,
}

#[derive(Hash, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum UniformPropertyType {
    // TODO: Use VertexFormat here
    Int,
    Float,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Struct(Vec<UniformProperty>),
    Array(Box<UniformPropertyType>, usize),
}

impl UniformPropertyType {
    pub fn get_size(&self) -> u64 {
        match self {
            UniformPropertyType::Int => 4,
            UniformPropertyType::Float => 4,
            UniformPropertyType::UVec4 => 4 * 4,
            UniformPropertyType::Vec2 => 4 * 2,
            UniformPropertyType::Vec3 => 4 * 3,
            UniformPropertyType::Vec4 => 4 * 4,
            UniformPropertyType::Mat3 => 4 * 4 * 3,
            UniformPropertyType::Mat4 => 4 * 4 * 4,
            UniformPropertyType::Struct(properties) => properties
                .iter()
                .map(|p| p.property_type.get_size())
                .fold(0, |total, size| total + size),
            UniformPropertyType::Array(property, length) => property.get_size() * *length as u64,
        }
    }
}

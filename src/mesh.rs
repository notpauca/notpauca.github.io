use std::{io::{BufReader, Cursor}, path::Path, collections::LinkedList};
use cgmath::{Matrix4, Rad, Vector3};
use wgpu::util::DeviceExt;
use crate::{fetch, Vertex};

pub type UnfinishedMesh = (Vec<Vertex>, Vec<[u16;3]>, Vector3<f32>, Vector3<Rad<f32>>, Vector3<f32>);

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<[u16; 3]>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub translation: Vector3<f32>,
    pub rotations: Vector3<Rad<f32>>,
    pub scale: Vector3<f32>,
    pub transform_uniform: MeshTransformUniform,
    pub transformation_matrix_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct MeshTransformUniform {
    inner: [[f32; 4]; 4]
}

impl Mesh {
    pub async fn obj_from_link(link: String, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Result<LinkedList<Self>, Box<dyn std::error::Error>> {
        let bin = fetch::binary_data(link).await.unwrap();
        let bin = Cursor::new(bin);
        let mut bin = BufReader::new(bin);

        //TODO: Load materials
        fn mat_load(_b: &Path) -> tobj::MTLLoadResult {
            Err(tobj::LoadError::GenericFailure)
        }

        let (model_vec, _material_list) = tobj::load_obj_buf(&mut bin, &tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() }, mat_load)?;

        let mut ret = LinkedList::new();
        for model in model_vec {
            let positions = model.mesh.positions.chunks(3).map(|thing| [thing[0], thing[1], thing[2]]);
            let vertex_coords = model.mesh.texcoords.chunks(2).map(|thing| [thing[0], thing[1]]);
            let normals = model.mesh.normals.chunks(3).map(|thing| [thing[0], thing[1], thing[2]]);
            let indices: Vec<[u16; 3]> = model.mesh.indices.chunks(3).map(|thing| [thing[0] as u16, thing[1] as u16, thing[2] as u16]).collect();

            let vertices: Vec<Vertex> = itertools::izip!(positions, vertex_coords, normals)
                .map(|(position, texture_coords, normal)| Vertex {position, texture_coords, normal}).collect();

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index buffer"),
                    contents: bytemuck::cast_slice(indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );

            let translation = Vector3::new(0.0, 0.0, 0.0);
            let rotations = Vector3::new(Rad(0.0), Rad(0.0), Rad(0.0));
            let scale = Vector3::new(0.0, 0.0, 0.0);

            let rotation_matrix = Matrix4::from_angle_x(rotations.x)
                * Matrix4::from_angle_y(rotations.y)
                * Matrix4::from_angle_z(rotations.z);

            let transform_matrix = Matrix4::from_translation(translation) * rotation_matrix * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
            let transform_uniform = MeshTransformUniform { inner: transform_matrix.into() };

            let transformation_matrix_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Transform matrix"),
                    contents: bytemuck::cast_slice(&[transform_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transformation_matrix_buffer.as_entire_binding(),
                    }
                ],
                label: Some("mesh_transform_bind_group"),
            });

            ret.push_back(Self {
                vertices,
                indices,
                vertex_buffer,
                index_buffer,
                translation,
                rotations,
                scale,
                transform_uniform,
                transformation_matrix_buffer,
                bind_group,
            })
        }
        Ok(ret)
    }


    pub fn _new((vertices, indices, translation, rotations, scale): UnfinishedMesh, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let rotation_matrix = Matrix4::from_angle_x(rotations.x)
            * Matrix4::from_angle_y(rotations.y)
            * Matrix4::from_angle_z(rotations.z);

        let transform_matrix = Matrix4::from_translation(translation) * rotation_matrix * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        let transform_uniform = MeshTransformUniform { inner: transform_matrix.into() };

        let transformation_matrix_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Transform matrix"),
                contents: bytemuck::cast_slice(&[transform_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transformation_matrix_buffer.as_entire_binding(),
                }
            ],
            label: Some("mesh_transform_bind_group"),
        });
        Self {
            vertices, indices, vertex_buffer, index_buffer, translation, rotations, scale, transform_uniform, transformation_matrix_buffer, bind_group
        }
    }

    pub fn update_uniform(&mut self) {
        let rotation_matrix = Matrix4::from_angle_x(self.rotations.x)
            * Matrix4::from_angle_y(self.rotations.y)
            * Matrix4::from_angle_z(self.rotations.z);
        let transform_matrix = Matrix4::from_translation(self.translation) * rotation_matrix * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.transform_uniform.inner = transform_matrix.into();
    }

    pub fn write_itself(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.transformation_matrix_buffer, 0, bytemuck::cast_slice(&[self.transform_uniform]));
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use euc::{buffer::Buffer2d, rasterizer::{BackfaceCullingDisabled, Triangles}, Interpolate, Pipeline};
use image::ImageFormat;
use regex::Regex;



fn main() -> Result<(), Box<dyn std::error::Error>> {
	let out_dir = std::env::var("OUT_DIR")?;
	let out_dir = Path::new(&out_dir);

	let suzanne_obj = File::open("src/suzanne.obj")?;
	let suzanne = Mesh::import_from_obj(BufReader::new(suzanne_obj))?;
	let suzanne_metadata = File::create(out_dir.join("suzanne.rs"))?;
	suzanne.export_metadata(256, 256, &mut BufWriter::new(suzanne_metadata))?;
	let suzanne_data = File::create(out_dir.join("suzanne.bin"))?;
	suzanne.export_data(&mut BufWriter::new(suzanne_data))?;
	let suzanne_face_map = File::create(out_dir.join("suzanne.map"))?;
	suzanne.export_face_map(256, 256, &mut BufWriter::new(suzanne_face_map))?;

	let bone_png = File::open("src/bone_adjusted.png")?;
	let bone = image::load(BufReader::new(bone_png), ImageFormat::Png)?.to_rgb8();
	let bone_metadata = File::create(out_dir.join("bone.rs"))?;
	let mut w = BufWriter::new(bone_metadata);
	writeln!(&mut w, "pub const WIDTH: usize = {};", bone.width())?;
	writeln!(&mut w, "pub const HEIGHT: usize = {};", bone.height())?;
	let bone_data = File::create(out_dir.join("bone.rgb"))?;
	let mut w = BufWriter::new(bone_data);
	w.write_all(&bone)?;

	Ok(())
}



struct Mesh {
	positions: Vec<[f32; 3]>,
	tex_coords: Vec<[f32; 2]>,
	normals: Vec<[f32; 3]>,
	vertices: Vec<[usize; 3]>,
	faces: Vec<[usize; 3]>
}

impl Mesh {
	fn import_from_obj<R: BufRead>(r: R) -> std::io::Result<Self> {
		let position_regex =
			Regex::new(r"^v (\-?\d+\.\d+) (\-?\d+\.\d+) (\-?\d+\.\d+)$").unwrap();
		let tex_coord_regex =
			Regex::new(r"^vt (\-?\d+\.\d+) (\-?\d+\.\d+)$").unwrap();
		let normal_regex =
			Regex::new(r"^vn (\-?\d+\.\d+) (\-?\d+\.\d+) (\-?\d+\.\d+)$").unwrap();
		let face_regex =
			Regex::new(r"^f (\d+)/(\d+)/(\d+) (\d+)/(\d+)/(\d+) (\d+)/(\d+)/(\d+)$").unwrap();

		let mut positions = Vec::new();
		let mut tex_coords = Vec::new();
		let mut normals = Vec::new();
		let mut vertices = Vec::new();
		let mut reverse_lookup = HashMap::new();
		let mut faces = Vec::new();

		for line in r.lines() {
			let line = line?;
			if let Some(captures) = position_regex.captures(&line) {
				positions.push([
					captures[1].parse().unwrap(),
					captures[2].parse().unwrap(),
					captures[3].parse().unwrap()
				]);
			} else if let Some(captures) = tex_coord_regex.captures(&line) {
				tex_coords.push([
					captures[1].parse().unwrap(),
					captures[2].parse().unwrap()
				]);
			} else if let Some(captures) = normal_regex.captures(&line) {
				normals.push([
					captures[1].parse().unwrap(),
					captures[2].parse().unwrap(),
					captures[3].parse().unwrap()
				]);
			} else if let Some(captures) = face_regex.captures(&line) {
				let mut face = [0, 1, 2];
				for v_out in face.iter_mut() {
					let vertex = [
						captures[3 * *v_out + 1].parse::<usize>().unwrap() - 1,
						captures[3 * *v_out + 2].parse::<usize>().unwrap() - 1,
						captures[3 * *v_out + 3].parse::<usize>().unwrap() - 1,
					];
					if let Some(v) = reverse_lookup.get(&vertex) {
						*v_out = *v;
					} else {
						vertices.push(vertex);
						*v_out = vertices.len() - 1;
						reverse_lookup.insert(vertex, *v_out);
					}
				}
				faces.push(face);
			}
		}

		Ok(Mesh { positions, tex_coords, normals, vertices, faces })
	}

	fn export_metadata<W: Write>(&self, width: usize, height: usize, w: &mut W) -> std::io::Result<()> {
		writeln!(w, "pub const VERTEX_COUNT: usize = {};", self.vertices.len())?;
		writeln!(w, "pub const FACE_COUNT: usize = {};", self.faces.len())?;
		writeln!(w, "pub const FACE_MAP_WIDTH: usize = {};", width)?;
		writeln!(w, "pub const FACE_MAP_HEIGHT: usize = {};", height)?;
		Ok(())
	}

	fn export_data<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
		let mut offset = 0;
		for vertex in self.vertices.iter() {
			w.write_all(bytemuck::bytes_of(&self.positions[vertex[0]]))?;
			offset += std::mem::size_of::<[f32; 3]>();
		}
		if offset % 256 != 0 {
			w.write_all(&vec![0u8; 256 - (offset % 256)])?;
			offset += 256 - (offset % 256);
		}
		for vertex in self.vertices.iter() {
			w.write_all(bytemuck::bytes_of(&self.tex_coords[vertex[1]]))?;
			offset += std::mem::size_of::<[f32; 2]>();
		}
		if offset % 256 != 0 {
			w.write_all(&vec![0u8; 256 - (offset % 256)])?;
		}
		for vertex in self.vertices.iter() {
			w.write_all(bytemuck::bytes_of(&self.normals[vertex[2]]))?;
		}

		let mut offset = 0;
		for face in self.faces.iter() {
			let indices = [face[0] as u32, face[1] as u32, face[2] as u32];
			w.write_all(bytemuck::bytes_of(&indices))?;
			offset += std::mem::size_of::<[u32; 3]>();
		}
		if offset % 256 != 0 {
			w.write_all(&vec![0u8; 256 - (offset % 256)])?;
		}
		for face in self.faces.iter() {
			let tc1 = self.tex_coords[self.vertices[face[0]][1]];
			let tc2 = self.tex_coords[self.vertices[face[1]][1]];
			let tc3 = self.tex_coords[self.vertices[face[2]][1]];
			let a = tc1[0] - tc3[0];
			let b = tc2[0] - tc3[0];
			let c = tc1[1] - tc3[1];
			let d = tc2[1] - tc3[1];
			let det = 1.0 / (a * d - b * c);
			let inv = [[det * d, det * -c], [det * -b, det * a]];
			w.write_all(bytemuck::bytes_of(&inv))?;
			w.write_all(bytemuck::bytes_of(&tc3))?;
		}
		Ok(())
	}

	fn export_face_map<W: Write>(&self, width: usize, height: usize, w: &mut W) -> std::io::Result<()> {
		let mut face_map = Buffer2d::new([width, height], !0);
		for (f, face) in self.faces.iter().enumerate() {
			FaceMap.draw::<Triangles<(f32,), BackfaceCullingDisabled>, _>(&[
				(self.tex_coords[self.vertices[face[0]][1]], f as u32),
				(self.tex_coords[self.vertices[face[1]][1]], f as u32),
				(self.tex_coords[self.vertices[face[2]][1]], f as u32)
			], &mut face_map, None);
		}
		w.write_all(bytemuck::cast_slice(face_map.as_ref()))?;
		Ok(())
	}
}

struct FaceMap;

impl Pipeline for FaceMap {
	type Vertex = ([f32; 2], u32);
	type VsOut = Face;
	type Pixel = u32;

	fn vert(&self, input: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
		([2.0 * input.0[0] - 1.0, 1.0 - 2.0 * input.0[1], 0.0, 1.0], Face(input.1))
	}

	fn frag(&self, input: &Self::VsOut) -> Self::Pixel {
		input.0
	}
}

#[derive(Clone, Copy)]
struct Face(u32);

impl Interpolate for Face {
	fn lerp2(a: Self, _: Self, _: f32, _: f32) -> Self { a }
	fn lerp3(a: Self, _: Self, _: Self, _: f32, _: f32, _: f32) -> Self { a }
}

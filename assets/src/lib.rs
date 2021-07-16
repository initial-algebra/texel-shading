mod suzanne {
	include!(concat!(env!("OUT_DIR"), "/suzanne.rs"));
}

pub const SUZANNE: Mesh<'static> = Mesh {
	vertex_count: suzanne::VERTEX_COUNT,
	face_count: suzanne::FACE_COUNT,
	bytes: include_bytes!(concat!(env!("OUT_DIR"), "/suzanne.bin"))
};

pub const SUZANNE_FACE_MAP: Texture<'static> = Texture {
	width: suzanne::FACE_MAP_WIDTH,
	height: suzanne::FACE_MAP_HEIGHT,
	bytes: include_bytes!(concat!(env!("OUT_DIR"), "/suzanne.map"))
};

mod bone {
	include!(concat!(env!("OUT_DIR"), "/bone.rs"));
}

pub const BONE: Texture<'static> = Texture {
	width: bone::WIDTH,
	height: bone::HEIGHT,
	bytes: include_bytes!(concat!(env!("OUT_DIR"), "/bone.rgb"))
};



pub struct Mesh<'a> {
	vertex_count: usize,
	face_count: usize,
	bytes: &'a [u8]
}

impl<'a> Mesh<'a> {
	pub const fn vertex_count(&self) -> usize { self.vertex_count }

	pub const fn positions_offset(&self) -> usize {
		0
	}

	pub const fn positions_size(&self) -> usize {
		self.vertex_count * std::mem::size_of::<[f32; 3]>()
	}

	pub const fn tex_coords_offset(&self) -> usize {
		align(self.positions_offset() + self.positions_size())
	}

	pub const fn tex_coords_size(&self) -> usize {
		self.vertex_count * std::mem::size_of::<[f32; 2]>()
	}

	pub const fn normals_offset(&self) -> usize {
		align(self.tex_coords_offset() + self.tex_coords_size())
	}

	pub const fn normals_size(&self) -> usize {
		self.vertex_count * std::mem::size_of::<[f32; 3]>()
	}

	pub const fn vertex_buffer_size(&self) -> usize {
		self.normals_offset() + self.normals_size()
	}

	pub fn vertex_buffer_bytes(&self) -> &[u8] {
		&self.bytes[0..self.vertex_buffer_size()]
	}

	pub const fn face_count(&self) -> usize { self.face_count }

	pub const fn faces_offset(&self) -> usize {
		0
	}

	pub const fn faces_size(&self) -> usize {
		self.face_count * std::mem::size_of::<[u32; 3]>()
	}

	pub const fn solvers_offset(&self) -> usize {
		align(self.faces_offset() + self.faces_size())
	}

	pub const fn solvers_size(&self) -> usize {
		self.face_count * std::mem::size_of::<[f32; 6]>()
	}

	pub const fn face_buffer_size(&self) -> usize {
		self.solvers_offset() + self.solvers_size()
	}

	pub fn face_buffer_bytes(&self) -> &[u8] {
		&self.bytes[self.vertex_buffer_size()..]
	}
}

const fn align(offset: usize) -> usize {
	if offset % 256 == 0 {
		offset
	} else {
		offset + (256 - offset % 256)
	}
}



pub struct Texture<'a> {
	width: usize,
	height: usize,
	bytes: &'a [u8]
}

impl<'a> Texture<'a> {
	pub const fn width(&self) -> usize { self.width }
	pub const fn height(&self) -> usize { self.height }
	pub const fn bytes(&self) -> &'a [u8] { self.bytes }
}

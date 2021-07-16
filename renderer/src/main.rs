use std::{ffi::CString, time::Instant};

use cgmath::{Deg, Matrix4, Point3, Vector3};
use gl::types::*;
use glutin::{Api, GlRequest, dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::run_return::EventLoopExtRunReturn, window::WindowBuilder};



fn main() {
	let mut event_loop = EventLoop::new();
	let builder = WindowBuilder::new()
		.with_inner_size(PhysicalSize { width: 1000, height: 1000 })
		.with_resizable(false)
		.with_title("Texture-space interpolation");
	let gl_window = glutin::ContextBuilder::new()
		.with_depth_buffer(32)
		.with_multisampling(4)
		.with_gl(GlRequest::Specific(Api::OpenGl, (4, 3)))
		.with_gl_profile(glutin::GlProfile::Core)
		.with_vsync(true)
		.build_windowed(builder, &event_loop).unwrap();
	let context = unsafe { gl_window.make_current() }.unwrap();
	gl::load_with(|symbol| context.get_proc_address(symbol));

	unsafe {
		gl::ClearColor(0.25, 0.25, 0.25, 1.0);
		gl::Enable(gl::DEPTH_TEST);
	}

	let object_program;
	unsafe {
		let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
		let vertex_glsl = CString::new(include_str!("object.vert")).unwrap();
		gl::ShaderSource(vertex_shader, 1, &vertex_glsl.as_ptr(), std::ptr::null());
		gl::CompileShader(vertex_shader);
		let mut status = 0;
		gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut status);
		if status as GLboolean == gl::FALSE {
			let mut log_len = 0;
			gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut log_len);
			let mut log_bytes = Vec::with_capacity(log_len as usize);
			gl::GetShaderInfoLog(
				vertex_shader, log_len, std::ptr::null_mut(), log_bytes.as_mut_ptr() as *mut GLchar
			);
			log_bytes.set_len(log_len as usize);
			let log = CString::from_vec_unchecked(log_bytes);
			panic!("{}", log.into_string().unwrap());
		}
		let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
		let fragment_glsl = CString::new(include_str!("object.frag")).unwrap();
		gl::ShaderSource(fragment_shader, 1, &fragment_glsl.as_ptr(), std::ptr::null());
		gl::CompileShader(fragment_shader);
		let mut status = 0;
		gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut status);
		if status as GLboolean == gl::FALSE {
			let mut log_len = 0;
			gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut log_len);
			let mut log_bytes = Vec::with_capacity(log_len as usize);
			gl::GetShaderInfoLog(
				fragment_shader, log_len, std::ptr::null_mut(), log_bytes.as_mut_ptr() as *mut GLchar
			);
			log_bytes.set_len(log_len as usize);
			let log = CString::from_vec_unchecked(log_bytes);
			panic!("{}", log.into_string().unwrap());
		}

		object_program = gl::CreateProgram();
		gl::AttachShader(object_program, vertex_shader);
		gl::AttachShader(object_program, fragment_shader);
		gl::LinkProgram(object_program);
		let mut status = 0;
		gl::GetProgramiv(object_program, gl::LINK_STATUS, &mut status);
		if status as GLboolean == gl::FALSE {
			let mut log_len = 0;
			gl::GetProgramiv(object_program, gl::INFO_LOG_LENGTH, &mut log_len);
			let mut log_bytes = Vec::with_capacity(log_len as usize);
			gl::GetProgramInfoLog(
				object_program, log_len, std::ptr::null_mut(), log_bytes.as_mut_ptr() as *mut GLchar
			);
			log_bytes.set_len(log_len as usize);
			let log = CString::from_vec_unchecked(log_bytes);
			panic!("{}", log.into_string().unwrap());
		}
	}

	let mut suzanne_vertex_array = 0;
	let mut suzanne_vertex_buffer = 0;
	let mut suzanne_face_buffer = 0;
	unsafe {
		gl::GenVertexArrays(1, &mut suzanne_vertex_array);
		gl::BindVertexArray(suzanne_vertex_array);

		gl::GenBuffers(1, &mut suzanne_vertex_buffer);
		gl::BindBuffer(gl::ARRAY_BUFFER, suzanne_vertex_buffer);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			assets::SUZANNE.vertex_buffer_size() as GLsizeiptr,
			assets::SUZANNE.vertex_buffer_bytes().as_ptr() as *const GLvoid,
			gl::STATIC_DRAW
		);
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(
			0, 3, gl::FLOAT, gl::FALSE, 0, assets::SUZANNE.positions_offset() as *const GLvoid
		);
		gl::EnableVertexAttribArray(1);
		gl::VertexAttribPointer(
			1, 2, gl::FLOAT, gl::FALSE, 0, assets::SUZANNE.tex_coords_offset() as *const GLvoid
		);
		gl::EnableVertexAttribArray(2);
		gl::VertexAttribPointer(
			2, 3, gl::FLOAT, gl::FALSE, 0, assets::SUZANNE.normals_offset() as *const GLvoid
		);

		gl::GenBuffers(1, &mut suzanne_face_buffer);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, suzanne_face_buffer);
		gl::BufferData(
			gl::ELEMENT_ARRAY_BUFFER,
			assets::SUZANNE.face_buffer_size() as GLsizeiptr,
			assets::SUZANNE.face_buffer_bytes().as_ptr() as *const GLvoid,
			gl::STATIC_DRAW
		);

		gl::BindVertexArray(0);
	}

	let mut suzanne_face_map = 0;
	unsafe {
		gl::GenTextures(1, &mut suzanne_face_map);
		gl::BindTexture(gl::TEXTURE_2D, suzanne_face_map);
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::R32UI as GLint,
			assets::SUZANNE_FACE_MAP.width() as GLsizei,
			assets::SUZANNE_FACE_MAP.height() as GLsizei,
			0,
			gl::RED_INTEGER,
			gl::UNSIGNED_INT,
			assets::SUZANNE_FACE_MAP.bytes().as_ptr() as *const GLvoid
		);
		gl::TexParameteri(
			gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint
		);
		gl::TexParameteri(
			gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint
		);
	}

	let mut bone_texture = 0;
	unsafe {
		gl::GenTextures(1, &mut bone_texture);
		gl::BindTexture(gl::TEXTURE_2D, bone_texture);
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::RGB as GLint,
			assets::BONE.width() as GLsizei,
			assets::BONE.height() as GLsizei,
			0,
			gl::RGB,
			gl::UNSIGNED_BYTE,
			assets::BONE.bytes().as_ptr() as *const GLvoid
		);
		gl::GenerateMipmap(gl::TEXTURE_2D);
		gl::TexParameteri(
			gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint
		);
		gl::TexParameteri(
			gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint
		);
	}

	let origin = Instant::now();
	event_loop.run_return(|event, _, control_flow| match event {
		Event::WindowEvent { event, window_id }
		if window_id == context.window().id() => match event {
			WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
			_ => ()
		},
		Event::MainEventsCleared => context.window().request_redraw(),
		Event::RedrawRequested(window_id) if window_id == context.window().id() => {
			let elapsed = (Instant::now() - origin).as_secs_f32();
			let projection = cgmath::perspective(Deg(45.0), 1.0, 0.001, 1000.0);
			let x = 4.0 * elapsed.sin();
			let z = 4.0 * elapsed.cos();
			let view = Matrix4::look_at_rh(
				Point3::new(x, 1.0, z),
				Point3::new(0.0, 0.0, 0.0),
				Vector3::new(0.0, 1.0, 0.0)
			);

			unsafe {
				gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

				gl::UseProgram(object_program);
				let projection_data: &[f32; 16] = projection.as_ref();
				gl::UniformMatrix4fv(0, 1, gl::FALSE, projection_data.as_ptr());
				let view_data: &[f32; 16] = view.as_ref();
				gl::UniformMatrix4fv(1, 1, gl::FALSE, view_data.as_ptr());
				gl::ActiveTexture(gl::TEXTURE0);
				gl::BindTexture(gl::TEXTURE_2D, suzanne_face_map);
				gl::ActiveTexture(gl::TEXTURE1);
				gl::BindTexture(gl::TEXTURE_2D, bone_texture);

				gl::BindBufferRange(
					gl::SHADER_STORAGE_BUFFER,
					0,
					suzanne_vertex_buffer,
					assets::SUZANNE.positions_offset() as GLintptr,
					assets::SUZANNE.positions_size() as GLsizeiptr
				);
				gl::BindBufferRange(
					gl::SHADER_STORAGE_BUFFER,
					1,
					suzanne_vertex_buffer,
					assets::SUZANNE.normals_offset() as GLintptr,
					assets::SUZANNE.normals_size() as GLsizeiptr
				);
				gl::BindBufferRange(
					gl::SHADER_STORAGE_BUFFER,
					2,
					suzanne_face_buffer,
					assets::SUZANNE.faces_offset() as GLintptr,
					assets::SUZANNE.faces_size() as GLsizeiptr
				);
				gl::BindBufferRange(
					gl::SHADER_STORAGE_BUFFER,
					3,
					suzanne_face_buffer,
					assets::SUZANNE.solvers_offset() as GLintptr,
					assets::SUZANNE.solvers_size() as GLsizeiptr
				);
				gl::BindVertexArray(suzanne_vertex_array);
				gl::DrawElements(
					gl::TRIANGLES,
					3 * assets::SUZANNE.face_count() as GLsizei,
					gl::UNSIGNED_INT,
					std::ptr::null()
				);
				gl::BindVertexArray(0);
			}
			context.swap_buffers().unwrap();
		}
		_ => ()
	});
}

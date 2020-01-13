use chrono::{Datelike, Timelike, Utc};
use glium::*;
use rusttype::gpu_cache::Cache;
use rusttype::{point, vector, Font, PositionedGlyph, Rect, Scale};
use std::borrow::Cow;
use std::error::Error;

fn layout_paragraph<'a>(
    font: &'a Font,
    scale: Scale,
    width: u32,
    text: &str,
) -> Vec<PositionedGlyph<'a>> {
    let mut result = Vec::new();
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = point(0.0, v_metrics.ascent);
    let mut last_glyph_id = None;
    for c in text.chars() {
        if c.is_control() {
            match c {
                '\r' => {
                    caret = point(0.0, caret.y + advance_height);
                }
                '\n' => {}
                _ => {}
            }
            continue;
        }
        let base_glyph = font.glyph(c);
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());
        let mut glyph = base_glyph.scaled(scale).positioned(caret);
        if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.max.x > width as i32 {
                caret = point(0.0, caret.y + advance_height);
                glyph.set_position(caret);
                last_glyph_id = None;
            }
        }
        caret.x += glyph.unpositioned().h_metrics().advance_width;
        result.push(glyph);
    }
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let font_data = include_bytes!("../fonts/digital-7 (mono).ttf");
    let font = Font::from_bytes(font_data as &[u8])?;

    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions((480, 220).into())
        .with_title("RustType GPU cache example");
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let mut events_loop = glium::glutin::EventsLoop::new();
    let display = glium::Display::new(window, context, &events_loop)?;

    let dpi_factor = display.gl_window().window().get_hidpi_factor();

    let (cache_width, cache_height) = ((512.0 * dpi_factor) as u32, (512.0 * dpi_factor) as u32);
    let mut cache = Cache::builder()
        .dimensions(cache_width, cache_height)
        .build();

    let program = program!(
    &display,
    140 => {
            vertex: "
                #version 140
                in vec2 position;
                in vec2 tex_coords;
                in vec4 colour;
                out vec2 v_tex_coords;
                out vec4 v_colour;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                    v_colour = colour;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                in vec4 v_colour;
                out vec4 f_colour;
                void main() {
                    f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
                }
            "
    })?;
    let cache_tex = glium::texture::Texture2d::with_format(
        &display,
        glium::texture::RawImage2d {
            data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
            width: cache_width,
            height: cache_height,
            format: glium::texture::ClientFormat::U8,
        },
        glium::texture::UncompressedFloatFormat::U8,
        glium::texture::MipmapsOption::NoMipmap,
    )?;

    loop {
        let now = Utc::now();
        let text: String = format!(
            "{:02}:{:02}:{:02}\r\r{:04}-{:02}-{:02}",
            now.hour(),
            now.minute(),
            now.second(),
            now.year(),
            now.month(),
            now.day()
        )
        .into();

        let dpi_factor = display.gl_window().window().get_hidpi_factor();
        let (width, _): (u32, _) = display
            .gl_window()
            .window()
            .get_inner_size()
            .ok_or("get_inner_size")?
            .to_physical(dpi_factor)
            .into();
        let dpi_factor = dpi_factor as f32;

        let mut finished = false;
        events_loop.poll_events(|event| {
            use glium::glutin::*;

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => finished = true,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keypress),
                                ..
                            },
                        ..
                    } => match keypress {
                        VirtualKeyCode::Escape => finished = true,
                        _ => (),
                    },
                    _ => {}
                }
            }
        });
        if finished {
            break;
        }

        let glyphs = layout_paragraph(&font, Scale::uniform(56.0 * dpi_factor), width, &text);
        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }
        cache.cache_queued(|rect, data| {
            cache_tex.main_level().write(
                glium::Rect {
                    left: rect.min.x,
                    bottom: rect.min.y,
                    width: rect.width(),
                    height: rect.height(),
                },
                glium::texture::RawImage2d {
                    data: Cow::Borrowed(data),
                    width: rect.width(),
                    height: rect.height(),
                    format: glium::texture::ClientFormat::U8,
                },
            );
        })?;

        let uniforms = uniform! {
            tex: cache_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                tex_coords: [f32; 2],
                colour: [f32; 4],
            }

            implement_vertex!(Vertex, position, tex_coords, colour);
            let colour = [1.0, 1.0, 1.0, 1.0];
            let (screen_width, screen_height) = {
                let (w, h) = display.get_framebuffer_dimensions();
                (w as f32, h as f32)
            };
            let origin = point(0.0, 0.0);
            let vertices: Vec<Vertex> = glyphs
                .iter()
                .flat_map(|g| {
                    if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
                        let gl_rect = Rect {
                            min: origin
                                + (vector(
                                    screen_rect.min.x as f32 / screen_width - 0.5,
                                    1.0 - screen_rect.min.y as f32 / screen_height - 0.5,
                                )) * 2.0,
                            max: origin
                                + (vector(
                                    screen_rect.max.x as f32 / screen_width - 0.5,
                                    1.0 - screen_rect.max.y as f32 / screen_height - 0.5,
                                )) * 2.0,
                        };
                        arrayvec::ArrayVec::<[Vertex; 6]>::from([
                            Vertex {
                                position: [gl_rect.min.x, gl_rect.max.y],
                                tex_coords: [uv_rect.min.x, uv_rect.max.y],
                                colour,
                            },
                            Vertex {
                                position: [gl_rect.min.x, gl_rect.min.y],
                                tex_coords: [uv_rect.min.x, uv_rect.min.y],
                                colour,
                            },
                            Vertex {
                                position: [gl_rect.max.x, gl_rect.min.y],
                                tex_coords: [uv_rect.max.x, uv_rect.min.y],
                                colour,
                            },
                            Vertex {
                                position: [gl_rect.max.x, gl_rect.min.y],
                                tex_coords: [uv_rect.max.x, uv_rect.min.y],
                                colour,
                            },
                            Vertex {
                                position: [gl_rect.max.x, gl_rect.max.y],
                                tex_coords: [uv_rect.max.x, uv_rect.max.y],
                                colour,
                            },
                            Vertex {
                                position: [gl_rect.min.x, gl_rect.max.y],
                                tex_coords: [uv_rect.min.x, uv_rect.max.y],
                                colour,
                            },
                        ])
                    } else {
                        arrayvec::ArrayVec::new()
                    }
                })
                .collect();

            glium::VertexBuffer::new(&display, &vertices)?
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(
            &vertex_buffer,
            glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &program,
            &uniforms,
            &glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
        )?;

        target.finish()?;
    }

    Ok(())
}

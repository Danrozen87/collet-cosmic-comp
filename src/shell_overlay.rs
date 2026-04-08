// SPDX-License-Identifier: GPL-3.0-only

//! Shell overlay -- renders the Collet Shell as a compositor-embedded layer.
//!
//! Currently renders a placeholder (dark background with dock bar and control
//! bar outlines). Will be replaced with WebView-rendered content.

use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            ImportMem, Renderer,
            element::{Kind, memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement}},
        },
    },
    output::Output,
    utils::Transform,
};

use crate::backend::render::element::{AsGlowRenderer, CosmicElement};

/// Cached shell buffer, regenerated when output dimensions change.
static SHELL_BUFFER: std::sync::Mutex<Option<(i32, i32, MemoryRenderBuffer)>> =
    std::sync::Mutex::new(None);

/// Create a shell overlay buffer with placeholder content.
fn create_shell_buffer(width: i32, height: i32) -> MemoryRenderBuffer {
    let w = width as usize;
    let h = height as usize;

    // ARGB8888 pixel buffer
    let mut pixels = vec![0u8; w * h * 4];

    // Dark background (oklch(0.13 0 0) ~ RGB(30, 30, 30))
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            // ARGB8888 byte order: B, G, R, A
            pixels[idx] = 30;      // B
            pixels[idx + 1] = 30;  // G
            pixels[idx + 2] = 30;  // R
            pixels[idx + 3] = 255; // A
        }
    }

    // Dock bar at bottom center
    let dock_w = 520.min(w);
    let dock_h = 60.min(h);
    let dock_x = (w.saturating_sub(dock_w)) / 2;
    let dock_y = h.saturating_sub(dock_h + 16);

    for y in dock_y..(dock_y + dock_h).min(h) {
        for x in dock_x..(dock_x + dock_w).min(w) {
            let idx = (y * w + x) * 4;
            pixels[idx] = 22;      // B
            pixels[idx + 1] = 22;  // G
            pixels[idx + 2] = 22;  // R
            pixels[idx + 3] = 230; // A
        }
    }

    // Control bar pill at top-right
    let bar_w = 250.min(w);
    let bar_h = 36.min(h);
    let bar_x = w.saturating_sub(bar_w + 14);
    let bar_y = 10.min(h.saturating_sub(bar_h));

    for y in bar_y..(bar_y + bar_h).min(h) {
        for x in bar_x..(bar_x + bar_w).min(w) {
            let idx = (y * w + x) * 4;
            pixels[idx] = 35;      // B
            pixels[idx + 1] = 35;  // G
            pixels[idx + 2] = 35;  // R
            pixels[idx + 3] = 200; // A
        }
    }

    // Thin bright line in dock to indicate content placeholder
    let text_y = dock_y + dock_h / 2;
    let text_x_start = dock_x + 80.min(dock_w / 2);
    let text_x_end = (dock_x + dock_w).saturating_sub(80.min(dock_w / 2));
    for x in text_x_start..text_x_end.min(w) {
        for dy in 0..2 {
            let y = text_y + dy;
            if y < h {
                let idx = (y * w + x) * 4;
                pixels[idx] = 180;     // B
                pixels[idx + 1] = 180; // G
                pixels[idx + 2] = 180; // R
                pixels[idx + 3] = 255; // A
            }
        }
    }

    MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Argb8888,
        (width, height),
        1,
        Transform::Normal,
        None,
    )
}

/// Render the shell overlay element for the given output.
///
/// Returns `None` if the output has no usable mode or the element cannot be
/// constructed (e.g. texture upload failure).
pub fn render_shell_element<R>(
    renderer: &mut R,
    output: &Output,
) -> Option<CosmicElement<R>>
where
    R: AsGlowRenderer,
    R::TextureId: Send + Clone + 'static,
{
    let output_size = output.current_mode()?.size;
    let w = output_size.w;
    let h = output_size.h;

    let buffer = {
        let mut cache = SHELL_BUFFER.lock().ok()?;
        match cache.as_ref() {
            Some((cw, ch, buf)) if *cw == w && *ch == h => buf.clone(),
            _ => {
                let buf = create_shell_buffer(w, h);
                *cache = Some((w, h, buf.clone()));
                buf
            }
        }
    };

    let element = MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        (0., 0.),
        &buffer,
        None,
        None,
        None,
        Kind::Unspecified,
    )
    .ok()?;

    Some(CosmicElement::Shell(element))
}

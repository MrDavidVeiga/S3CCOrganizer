use std::{fs, io, path::Path};

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn generated_manager_icon() -> Vec<u8> {
    const SIZE: usize = 64;
    const MASK_ROW: usize = 8; // 64 bits, already DWORD aligned.
    const PIXEL_BYTES: usize = SIZE * SIZE * 4;
    const IMAGE_BYTES: usize = 40 + PIXEL_BYTES + MASK_ROW * SIZE;

    let mut out = Vec::with_capacity(22 + IMAGE_BYTES);

    // ICONDIR
    push_u16(&mut out, 0);
    push_u16(&mut out, 1);
    push_u16(&mut out, 1);

    // ICONDIRENTRY
    out.push(SIZE as u8);
    out.push(SIZE as u8);
    out.push(0);
    out.push(0);
    push_u16(&mut out, 1);
    push_u16(&mut out, 32);
    push_u32(&mut out, IMAGE_BYTES as u32);
    push_u32(&mut out, 22);

    // BITMAPINFOHEADER. ICO DIB height includes XOR + AND masks.
    push_u32(&mut out, 40);
    push_u32(&mut out, SIZE as u32);
    push_u32(&mut out, (SIZE * 2) as u32);
    push_u16(&mut out, 1);
    push_u16(&mut out, 32);
    push_u32(&mut out, 0);
    push_u32(&mut out, PIXEL_BYTES as u32);
    push_u32(&mut out, 0);
    push_u32(&mut out, 0);
    push_u32(&mut out, 0);
    push_u32(&mut out, 0);

    // BGRA, bottom-up. Draw a compact green plumbob/diamond.
    for stored_y in 0..SIZE {
        let y = SIZE - 1 - stored_y;
        for x in 0..SIZE {
            let cx = x as i32 - 32;
            let cy = y as i32 - 32;
            let vertical = cy.abs();
            let half_width = if vertical <= 28 { 26 - (vertical * 24 / 28) } else { -1 };
            let inside = half_width >= 0 && cx.abs() <= half_width;
            let edge = inside && cx.abs() >= half_width.saturating_sub(2);

            let (b, g, r, a) = if edge {
                (28u8, 70u8, 33u8, 255u8)
            } else if inside {
                let light = ((28 - vertical).max(0) * 3) as u8;
                (
                    42u8.saturating_add(light / 5),
                    145u8.saturating_add(light),
                    67u8.saturating_add(light / 2),
                    255u8,
                )
            } else {
                (0, 0, 0, 0)
            };
            out.extend_from_slice(&[b, g, r, a]);
        }
    }

    // AND mask: transparent state is already carried by alpha.
    out.resize(out.len() + MASK_ROW * SIZE, 0);
    out
}

fn ensure_icon() -> io::Result<()> {
    let path = Path::new("icons").join("icon.ico");
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, generated_manager_icon())
}

fn main() {
    ensure_icon().expect("failed to prepare Windows application icon");
    tauri_build::build()
}

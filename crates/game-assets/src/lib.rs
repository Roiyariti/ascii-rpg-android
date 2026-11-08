//! Atlas original y pequeño, sin recursos externos ni acceso al sistema de archivos.
pub const ATLAS_WIDTH: u32 = 128;
pub const ATLAS_HEIGHT: u32 = 64;
pub const CELL: u32 = 8;

pub fn glyph_index(character: char) -> u32 {
    if character.is_ascii_graphic() || character == ' ' {
        character as u32
    } else {
        '?' as u32
    }
}

/// Genera una textura R8 de 8 KiB a partir de los trazos originales 5x7.
pub fn atlas() -> Vec<u8> {
    let mut pixels = vec![0; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
    for line in include_str!("../../../assets/font5x7.txt").lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut fields = line.split_whitespace();
        let index: u32 = fields
            .next()
            .expect("código ASCII")
            .parse()
            .expect("entero");
        for (y, row) in fields.enumerate() {
            let bits = u8::from_str_radix(row, 16).expect("fila hexadecimal");
            for x in 0..5 {
                if bits & (1 << (4 - x)) != 0 {
                    let px = index % 16 * CELL + x + 1;
                    let py = index / 16 * CELL + y as u32;
                    pixels[(py * ATLAS_WIDTH + px) as usize] = 255;
                }
            }
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printable_ascii_has_unique_entries_and_visible_ink_except_space() {
        let mut seen = [false; 128];
        for line in include_str!("../../../assets/font5x7.txt").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split_whitespace().collect();
            assert_eq!(fields.len(), 8);
            let index: usize = fields[0].parse().unwrap();
            assert!((32..=126).contains(&index));
            assert!(!seen[index]);
            seen[index] = true;
            let rows: Vec<_> = fields[1..]
                .iter()
                .map(|r| u8::from_str_radix(r, 16).unwrap())
                .collect();
            assert!(rows.iter().all(|&r| r < 32));
            assert_eq!(rows.iter().any(|&r| r != 0), index != 32);
        }
        assert!(seen[32..=126].iter().all(|&s| s));
        assert_eq!(atlas().len(), 8192);
        assert_eq!(glyph_index('ñ'), glyph_index('?'));
    }
}

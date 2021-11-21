use glam::Vec4;

pub fn decompress_coefficients(unk5: f32, unk6: f32, compressed_coefficients: [u8; 4]) -> [f32; 4] {
    // Reverse the coefficients to match how they appear in the uniform buffer.
    // TODO: Skip the reversing?
    let t = Vec4::new(
        compressed_coefficients[3] as f32 / 255.0,
        compressed_coefficients[2] as f32 / 255.0,
        compressed_coefficients[1] as f32 / 255.0,
        compressed_coefficients[0] as f32 / 255.0,
    );

    // TODO: The sign of unk5 and unk6 is sometimes flipped by unk2s?
    let min_value = Vec4::new(0.1481, -0.2962, -0.08551, 0.35544)
        + Vec4::new(-0.32573, -0.32573, -0.32573, 0.28209) * unk5;
    let scale = 71.93411 * unk6;
    (min_value + t * scale).to_array()
}

// TODO: Define the inverse operation?
fn compress_coefficients(unk5: f32, unk6: f32, coefficients: [f32; 4]) -> [u8; 4] {
    [0u8; 4]
}

#[cfg(test)]
mod tests {
    use crate::decompress_coefficients;
    use approx::relative_eq;

    macro_rules! assert_almost_eq {
        ($a:expr, $b:expr) => {
            assert!(
                relative_eq!($a.as_ref(), $b.as_ref(), epsilon = 0.0001),
                "{:?} != {:?}",
                $a,
                $b
            );
        };
    }

    // TODO: The only calculations that are accurate for now set unk5 to 0.
    // Test values are taken from cbuf11[19], cbuf11[20], and cbuf11[21].
    // The vertex shader uses these vectors to calculate RGB diffuse lighting.
    #[test]
    fn test1() {
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 72.28955],
            decompress_coefficients(0.0, 1.0, [255, 0, 0, 0])
        );

        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 36.46354],
            decompress_coefficients(0.0, 1.0, [128, 0, 0, 0])
        );

        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 1.0, [0, 0, 0, 0])
        );
    }

    #[test]
    fn test2() {
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 144.22372],
            decompress_coefficients(0.0, 2.0, [255, 0, 0, 0])
        );

        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 72.57165],
            decompress_coefficients(0.0, 2.0, [128, 0, 0, 0])
        );

        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 2.0, [0, 0, 0, 0])
        );
    }
}

use glam::Vec4;

// Constants were determined experimentally from the uniform buffer cbuf11 in Yuzu emulator.
// Values at index 19, 20, and 21 contain the red, green, and blue coefficients.
// The vertex shader uses these vectors to calculate RGB ambient diffuse lighting.
const SH_MIN: [f32; 4] = [0.1481, -0.2962, -0.08551, 0.35544];
const SH_MIN_SCALE: [f32; 4] = [0.32573, 0.32573, 0.32573, 0.28209];
const SH_MAX_SCALE: f32 = 71.93413;

pub fn decompress_coefficients(unk5: f32, unk6: f32, compressed_coefficients: [u8; 4]) -> [f32; 4] {
    // Reverse the coefficients to match how they appear in the uniform buffer.
    // TODO: Skip the reversing?
    let t = Vec4::new(
        compressed_coefficients[3] as f32 / 255.0,
        compressed_coefficients[2] as f32 / 255.0,
        compressed_coefficients[1] as f32 / 255.0,
        compressed_coefficients[0] as f32 / 255.0,
    );

    let min_value = Vec4::from(SH_MIN) + Vec4::from(SH_MIN_SCALE) * unk5;
    let scale = SH_MAX_SCALE * unk6;
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

    #[test]
    fn decompress_sh_coefficients() {
        assert_almost_eq!(
            [41.51643, 41.07212, 41.28282, 0.07334],
            decompress_coefficients(-1.0, 1.0, [0, 128, 128, 128])
        );
        assert_almost_eq!(
            [-0.17763, -0.62194, -0.41124, 0.07336],
            decompress_coefficients(-1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.47383, 0.02954, 0.24023, 36.74563],
            decompress_coefficients(1.0, 1.0, [128, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 144.22372],
            decompress_coefficients(0.0, 2.0, [255, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17762, -0.62195, -0.41125, 0.07337],
            decompress_coefficients(-1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 2.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.47383, 0.02954, 0.24023, 72.57165],
            decompress_coefficients(1.0, 1.0, [255, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17764, -0.62193, -0.41124, 36.18145],
            decompress_coefficients(1.0, 1.0, [128, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 36.46354],
            decompress_coefficients(0.0, 1.0, [128, 0, 0, 0])
        );
        assert_almost_eq!(
            [82.88475, 82.44044, 82.65115, 0.07334],
            decompress_coefficients(-1.0, 1.0, [0, 255, 255, 255])
        );
        assert_almost_eq!(
            [-0.17764, -0.62193, -0.41124, 72.00748],
            decompress_coefficients(1.0, 1.0, [255, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17764, -0.62193, -0.41124, 0.07334],
            decompress_coefficients(-1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17763, -0.62194, -0.41124, 72.00747],
            decompress_coefficients(-1.0, 1.0, [255, 0, 0, 0])
        );
        assert_almost_eq!(
            [82.88475, 82.44044, 82.65115, 72.00748],
            decompress_coefficients(-1.0, 1.0, [255, 255, 255, 255])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 72.57165],
            decompress_coefficients(0.0, 2.0, [128, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.47383, 0.02954, 0.24023, 0.63753],
            decompress_coefficients(1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17763, -0.62194, -0.41124, 0.07335],
            decompress_coefficients(-1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 72.28955],
            decompress_coefficients(0.0, 1.0, [255, 0, 0, 0])
        );
        assert_almost_eq!(
            [-0.17763, -0.62194, -0.41124, 36.18145],
            decompress_coefficients(-1.0, 1.0, [128, 0, 0, 0])
        );
    }
}

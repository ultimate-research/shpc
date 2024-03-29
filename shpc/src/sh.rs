//! Functions for compressing and decompressing spherical harmonic coefficients.
use glam::{const_vec4, Vec4};

// Constants were determined experimentally from the uniform buffer vp_c11 in Ryujinx emulator.
// An example of the buffer output from debugging with RenderDoc.
// vp_c11[19] 0.1481, -0.2962, -0.08551, 0.35544 float4
// vp_c11[20] 0.1481, -0.2962, -0.08551, 0.35544 float4
// vp_c11[21] 0.1481, -0.2962, -0.08551, 0.35544 float4
// TODO: Should it be possible for decompress -> compress -> decompress to be 1:1 given the low precision (8-bit)?
const SH_MIN: Vec4 = const_vec4!([0.1481, -0.2962, -0.08551, 0.35544]);
const SH_SCALE: Vec4 = const_vec4!([0.32573469, 0.32573469, 0.32573469, 0.28209451]);

// TODO: Investigate why the coefficients in game can sometimes be nan.
pub fn decompress_coefficients(unk5: f32, unk6: f32, compressed_coefficients: [u8; 4]) -> [f32; 4] {
    // Reverse the coefficients to match how they appear in the uniform buffer.
    // TODO: Skip the reversing?
    let t = Vec4::new(
        compressed_coefficients[3] as f32,
        compressed_coefficients[2] as f32,
        compressed_coefficients[1] as f32,
        compressed_coefficients[0] as f32,
    );

    let min_value = SH_MIN + SH_SCALE * unk5;
    let scale = SH_SCALE * unk6;
    (t * scale + min_value).to_array()
}

pub fn compress_coefficients(unk5: f32, unk6: f32, coefficients: [f32; 4]) -> [u8; 4] {
    let t = Vec4::from(coefficients);

    let min_value = SH_MIN + SH_SCALE * unk5;

    // When unk6 is zero, the result doesn't depend on the buffer values.
    // We'll just a buffer of all zeros to avoid division by zero.
    let scale = SH_SCALE * unk6;
    let buffer = if unk6 != 0.0 {
        (t - min_value) / scale
    } else {
        Vec4::ZERO
    };

    // Rounding makes the conversion more robust to rounding errors and innacurate constants.
    // Reverse the coefficients to match how they appear in the shpcanim file.
    // TODO: Skip the reversing?
    let [b3, b2, b1, b0] = buffer.round().to_array();
    [b0 as u8, b1 as u8, b2 as u8, b3 as u8]
}

#[cfg(test)]
mod tests {
    use super::*;
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
        // TODO: Put the coefficients first?
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 0.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 0.0, [128, 128, 128, 128])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35544],
            decompress_coefficients(0.0, 0.0, [255, 255, 255, 255])
        );
        assert_almost_eq!(
            [0.1481, -0.29621, -0.08551, 0.35545],
            decompress_coefficients(0.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [41.84214, 41.39783, 41.60854, 36.46354],
            decompress_coefficients(0.0, 1.0, [128, 128, 128, 128])
        );
        assert_almost_eq!(
            [83.21046, 82.76614, 82.97684, 72.28955],
            decompress_coefficients(0.0, 1.0, [255, 255, 255, 255])
        );
        assert_almost_eq!(
            [-0.17763, -0.62194, -0.41124, 0.0733],
            decompress_coefficients(-1.0, 1.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [41.51643, 41.07212, 41.28282, 36.18145],
            decompress_coefficients(-1.0, 1.0, [128, 128, 128, 128])
        );
        assert_almost_eq!(
            [82.88475, 82.44044, 82.65115, 72.00748],
            decompress_coefficients(-1.0, 1.0, [255, 255, 255, 255])
        );
        assert_almost_eq!(
            [0.1481, -0.2962, -0.08551, 0.35545],
            decompress_coefficients(0.0, 2.0, [0, 0, 0, 0])
        );
        assert_almost_eq!(
            [83.53618, 83.09192, 83.30261, 72.57165],
            decompress_coefficients(0.0, 2.0, [128, 128, 128, 128])
        );
        assert_almost_eq!(
            [166.27286, 165.82857, 166.03926, 144.22372],
            decompress_coefficients(0.0, 2.0, [255, 255, 255, 255])
        );
    }

    #[test]
    fn compress_sh_coefficients() {
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(1.0, 1.0, [0.47383, 0.02954, 0.24023, 0.63753])
        );
        assert_eq!(
            [255, 0, 0, 0],
            compress_coefficients(1.0, 1.0, [0.47383, 0.02954, 0.24023, 72.57165])
        );
        assert_eq!(
            [128, 0, 0, 0],
            compress_coefficients(1.0, 1.0, [0.47383, 0.02954, 0.24023, 36.74563])
        );
        assert_eq!(
            [255, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17763, -0.62194, -0.41124, 72.00747])
        );
        assert_eq!(
            [128, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17763, -0.62194, -0.41124, 36.18145])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17763, -0.62194, -0.41124, 0.07335])
        );
        assert_eq!(
            [255, 255, 255, 255],
            compress_coefficients(-1.0, 1.0, [82.88475, 82.44044, 82.65115, 72.00748])
        );
        assert_eq!(
            [0, 255, 255, 255],
            compress_coefficients(-1.0, 1.0, [82.88475, 82.44044, 82.65115, 0.07334])
        );
        assert_eq!(
            [0, 128, 128, 128],
            compress_coefficients(-1.0, 1.0, [41.51643, 41.07212, 41.28282, 0.07334])
        );
        assert_eq!(
            [255, 0, 0, 0],
            compress_coefficients(0.0, 1.0, [0.1481, -0.2962, -0.08551, 72.28955])
        );
        assert_eq!(
            [128, 0, 0, 0],
            compress_coefficients(0.0, 1.0, [0.1481, -0.2962, -0.08551, 36.46354])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(0.0, 1.0, [0.1481, -0.2962, -0.08551, 0.35544])
        );
        assert_eq!(
            [255, 0, 0, 0],
            compress_coefficients(0.0, 2.0, [0.1481, -0.2962, -0.08551, 144.22372])
        );
        assert_eq!(
            [128, 0, 0, 0],
            compress_coefficients(0.0, 2.0, [0.1481, -0.2962, -0.08551, 72.57165])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(0.0, 2.0, [0.1481, -0.2962, -0.08551, 0.35544])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17764, -0.62193, -0.41124, 0.07334])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17762, -0.62195, -0.41125, 0.07337])
        );
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(-1.0, 1.0, [-0.17763, -0.62194, -0.41124, 0.07336])
        );
        assert_eq!(
            [255, 255, 255, 255],
            compress_coefficients(1.0, 1.0, [83.53618, 83.09192, 83.30261, 72.57165])
        );
        assert_eq!(
            [255, 255, 255, 255],
            compress_coefficients(0.0, 2.0, [166.27257, 165.82825, 166.03894, 144.22346])
        );
    }

    #[test]
    fn compress_sh_coefficients_unk5_unk6_zeros() {
        // The case when unk6 is zero is ambiguous,
        // Any buffer values will decompress to these coefficients.
        // We'll just pick zero for now.
        assert_eq!(
            [0, 0, 0, 0],
            compress_coefficients(0.0, 0.0, [0.1481, -0.2962, -0.08551, 0.35544])
        );
    }
}

use shan::{Grid, Shan, Tpcb, TpcbHeader};
use ssbh_lib::Ptr32;

pub mod sh;
pub mod shan;

// TODO: Create a higher level API for applications to use.
#[derive(Debug, Clone, PartialEq)]
pub struct ShanFile {
    pub name: String,
    pub tpcbs: Vec<TpcbData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TpcbData {
    // TODO: Use a count so the starting frame list is non decreasing.
    pub starting_frame: u32,
    pub coefficients: GridCoefficients,
}

// TODO: We can recreate the grid attributes from a smaller set of attributes.
// grid_cell_count_xyz, grid_range_min, grid_range_max
// Return an error if the counts don't match the supplied coefficients?
#[derive(Debug, Clone, PartialEq)]
pub struct GridCoefficients {
    // TODO: Should this be immutable?
    pub grid_cell_count_xyz: [u32; 3],
    pub grid_range_min_xyz: [f32; 3],
    pub grid_range_max_xyz: [f32; 3],
    // TODO: These values are tied to the coefficients and shouldn't be editable?
    pub unk5: f32,
    pub unk6: f32,

    // TODO: Keep this private so people don't try to index manually?
    // TODO: The length should not exceed the capacity of u16 (used for indices)
    pub coefficients: Vec<[[f32; 4]; 3]>,
}

impl GridCoefficients {
    // TODO: Add trilinear interpolation?
    pub fn get(&self, x: usize, y: usize, z: usize) -> [f32; 3] {
        [0.0; 3]
    }
}

impl From<&Shan> for ShanFile {
    fn from(shan: &Shan) -> Self {
        // TODO: Avoid unwrap.
        Self {
            name: shan.name.to_string_lossy(),
            tpcbs: shan
                .tpcbs
                .iter()
                .zip(shan.tpcb_starting_frames.iter())
                .map(|(tpcb, starting_frame)| TpcbData {
                    starting_frame: *starting_frame,
                    coefficients: tpcb.as_ref().unwrap().into(),
                })
                .collect(),
        }
    }
}

impl From<&ShanFile> for Shan {
    fn from(shan: &ShanFile) -> Self {
        Self {
            unk1: shan
                .tpcbs
                .iter()
                .map(|t| t.starting_frame)
                .max()
                .unwrap_or_default(),
            tpcb_count: shan.tpcbs.len() as u32,
            unk3: 0,
            name: shan.name.clone().into(),
            tpcb_starting_frames: shan.tpcbs.iter().map(|t| t.starting_frame).collect(),
            tpcbs: shan
                .tpcbs
                .iter()
                .map(|tpcb| Ptr32::new((&tpcb.coefficients).into()))
                .collect(),
        }
    }
}

// TODO: Also implement for non references.
impl From<&Tpcb> for GridCoefficients {
    fn from(t: &Tpcb) -> Self {
        Self {
            grid_cell_count_xyz: t.inner.header.grid_cell_count_xyz,
            grid_range_min_xyz: t.inner.header.grid_range_min_xyz,
            grid_range_max_xyz: t.inner.header.grid_range_max_xyz,
            unk5: t.inner.header.unk5,
            unk6: t.inner.header.unk6,
            coefficients: t
                .inner
                .grid_sh_coefficients
                .0
                .as_ref()
                .unwrap()
                .iter()
                .map(|c| {
                    [
                        sh::decompress_coefficients(t.inner.header.unk5, t.inner.header.unk6, c.r),
                        sh::decompress_coefficients(t.inner.header.unk5, t.inner.header.unk6, c.g),
                        sh::decompress_coefficients(t.inner.header.unk5, t.inner.header.unk6, c.b),
                    ]
                })
                .collect(),
        }
    }
}

impl From<&GridCoefficients> for Tpcb {
    fn from(g: &GridCoefficients) -> Self {
        // TODO: Is there a cleaner way of calculating this?
        let mut grid_dimensions_xyz = [0.0; 3];
        for i in 0..3 {
            grid_dimensions_xyz[i] = g.grid_range_max_xyz[i] - g.grid_range_min_xyz[i];
        }

        let mut grid_spacing_xyz = [1.0; 3];
        for i in 0..3 {
            if g.grid_cell_count_xyz[i] > 1 {
                grid_spacing_xyz[i] =
                    grid_dimensions_xyz[i] / (g.grid_cell_count_xyz[i] as f32 - 1.0);
            }
        }

        Self {
            inner: shan::TpcbInner {
                header: TpcbHeader {
                    unk1_1: 1,
                    unk1_2: 35, // TODO: How to fill in this value?
                    grid_cell_count_xyz: g.grid_cell_count_xyz,
                    grid_spacing_xyz,
                    grid_dimensions_xyz,
                    grid_range_min_xyz: g.grid_range_min_xyz,
                    grid_range_max_xyz: g.grid_range_max_xyz,
                    unk4: 12,
                    unk5: g.unk5,
                    unk6: g.unk6,
                    grid_cell_count: g.coefficients.len() as u32,
                },
                grid_indices: Grid(Some((0..g.coefficients.len() as u16).collect())),
                grid_sh_coefficients: Grid(None),
                grid_unk_values: Grid(None),
            },
        }
    }
}

// TODO: Tests for this based on existing files.
// TODO: Don't test coefficients for now due to rounding errors?
#[cfg(test)]
mod tests {
    use ssbh_lib::Ptr32;

    use super::*;
    use crate::shan::{Grid, Shan, TpcbHeader, TpcbInner};

    #[test]
    fn shan_file_xeno_gaur() {
        // stage/xeno_gaur/normal/render/chara.shpcanim
        // Just test the overall structure and frame counts.
        let shan = Shan {
            unk1: 7200,
            tpcb_count: 9,
            unk3: 0,
            name: String::new().into(),
            tpcb_starting_frames: vec![0, 2800, 3000, 3400, 3600, 6400, 6600, 7000, 7200],
            tpcbs: vec![
                Ptr32::new(Tpcb {
                    inner: TpcbInner {
                        header: TpcbHeader {
                            unk1_1: 1,
                            unk1_2: 3,
                            grid_cell_count_xyz: [0; 3],
                            grid_spacing_xyz: [0.0; 3],
                            grid_dimensions_xyz: [0.0; 3],
                            grid_range_min_xyz: [0.0; 3],
                            grid_range_max_xyz: [0.0; 3],
                            unk4: 12,
                            unk5: -1.0247978,
                            unk6: 0.0313374,
                            grid_cell_count: 21,
                        },
                        grid_indices: Grid(None),
                        grid_sh_coefficients: Grid(None),
                        grid_unk_values: Grid(None),
                    },
                });
                9
            ],
        };

        let coefficients = GridCoefficients {
            grid_cell_count_xyz: [0; 3],
            grid_range_min_xyz: [0.0; 3],
            grid_range_max_xyz: [0.0; 3],
            unk5: -1.0247978,
            unk6: 0.0313374,
            coefficients: Vec::new(),
        };
        let shan_file = ShanFile {
            name: String::new(),
            tpcbs: vec![
                TpcbData {
                    starting_frame: 0,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 2800,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 3000,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 3400,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 3600,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 6400,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 6600,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    starting_frame: 7000,
                    coefficients: coefficients.clone(),
                },
                TpcbData {
                    // TODO: What is the length of the last tcpb?
                    starting_frame: 7200,
                    coefficients: coefficients.clone(),
                },
            ],
        };

        let new_shan_file = ShanFile::from(&shan);
        assert_eq!(new_shan_file, shan_file);

        let new_shan = Shan::from(&shan_file);
        assert_eq!(new_shan.unk1, shan.unk1);
        assert_eq!(new_shan.tpcb_count, shan.tpcb_count);
        assert_eq!(new_shan.unk3, shan.unk3);
        assert_eq!(new_shan.name, shan.name);
        assert_eq!(new_shan.tpcb_starting_frames, shan.tpcb_starting_frames);
        // TODO: Test other fields.
    }

    #[test]
    fn grid_coefficients_training() {
        // stage/training/normal/render/chara.shpcanim
        let tpcb = Tpcb {
            inner: TpcbInner {
                header: TpcbHeader {
                    unk1_1: 1,
                    unk1_2: 35,
                    grid_cell_count_xyz: [21, 10, 1],
                    grid_spacing_xyz: [31.444525, 25.039896, 1.0],
                    grid_dimensions_xyz: [628.8905, 225.35907, 0.0],
                    grid_range_min_xyz: [-563.37305, -98.03044, 0.0],
                    grid_range_max_xyz: [65.51749, 127.32863, 0.0],
                    unk4: 12,
                    unk5: -1.2438285,
                    unk6: 0.020140974,
                    grid_cell_count: 210,
                },
                grid_indices: Grid(Some(vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
                    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
                    62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81,
                    82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
                    101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
                    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132,
                    133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
                    149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164,
                    165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180,
                    181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196,
                    197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209,
                ])),
                grid_sh_coefficients: Grid(None),
                grid_unk_values: Grid(None),
            },
        };

        let grid = GridCoefficients {
            grid_cell_count_xyz: [21, 10, 1],
            grid_range_min_xyz: [-563.37305, -98.03044, 0.0],
            grid_range_max_xyz: [65.51749, 127.32863, 0.0],
            unk5: -1.2438285,
            unk6: 0.020140974,
            coefficients: vec![[[0.0; 4]; 3]; 210],
        };

        // Test GridCoefficients -> Tpcb
        let new_tpcb = Tpcb::from(&grid);
        assert_eq!(new_tpcb.inner.header, tpcb.inner.header);
        assert_eq!(new_tpcb.inner.grid_indices.0, tpcb.inner.grid_indices.0);
        // TODO: Test coefficient lengths?

        // Test Tpcb -> GridCoefficients
        let new_grid = GridCoefficients::from(&tpcb);
        assert_eq!(new_grid.grid_cell_count_xyz, grid.grid_cell_count_xyz);
        assert_eq!(new_grid.grid_range_min_xyz, grid.grid_range_min_xyz);
        assert_eq!(new_grid.grid_range_min_xyz, grid.grid_range_min_xyz);
        // TODO: Test coefficient lengths?
    }

    #[test]
    fn grid_coefficients_xeno_gaur() {
        // stage/xeno_gaur/normal/render/chara.shpcanim
        let tpcb = Tpcb {
            inner: TpcbInner {
                header: TpcbHeader {
                    unk1_1: 1,
                    unk1_2: 3, // TODO: These aren't preserved properly.
                    grid_cell_count_xyz: [0, 0, 0],
                    grid_spacing_xyz: [1.0, 1.0, 1.0],
                    grid_dimensions_xyz: [0.0, 0.0, 0.0],
                    grid_range_min_xyz: [0.0, 0.0, 0.0],
                    grid_range_max_xyz: [0.0, 0.0, 0.0],
                    unk4: 12,
                    unk5: -1.0247978,
                    unk6: 0.0313374,
                    grid_cell_count: 21,
                },
                grid_indices: Grid(Some(vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                ])),
                grid_sh_coefficients: Grid(None),
                grid_unk_values: Grid(None),
            },
        };

        let grid = GridCoefficients {
            grid_cell_count_xyz: [0, 0, 0],
            grid_range_min_xyz: [0.0, 0.0, 0.0],
            grid_range_max_xyz: [0.0, 0.0, 0.0],
            unk5: -1.0247978,
            unk6: 0.0313374,
            coefficients: vec![[[0.0; 4]; 3]; 21],
        };

        // Test GridCoefficients -> Tpcb
        let new_tpcb = Tpcb::from(&grid);
        assert_eq!(new_tpcb.inner.header, tpcb.inner.header);
        assert_eq!(new_tpcb.inner.grid_indices.0, tpcb.inner.grid_indices.0);
        // TODO: Test coefficient lengths?

        // Test Tpcb -> GridCoefficients
        let new_grid = GridCoefficients::from(&tpcb);
        assert_eq!(new_grid.grid_cell_count_xyz, grid.grid_cell_count_xyz);
        assert_eq!(new_grid.grid_range_min_xyz, grid.grid_range_min_xyz);
        assert_eq!(new_grid.grid_range_min_xyz, grid.grid_range_min_xyz);
        // TODO: Test coefficient lengths?
    }
}

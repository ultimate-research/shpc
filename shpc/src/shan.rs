//! The low level API for SHAN files stored in .shpcanim or .shpc files.
use binrw::{binread, prelude::*, PosValue};
use ssbh_lib::Ptr32;
use ssbh_write::SsbhWrite;
use std::fmt::Debug;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// TODO: Can this all be done without an ssbh_lib dependency and use binrw?
// There aren't very many offsets to calculate.
/// A Spherical Harmonic ANimation (SHAN) file like chara.shpcanim.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, BinRead, SsbhWrite, PartialEq, Clone)]
#[br(magic(b"SHAN"))]
#[ssbhwrite(magic = b"SHAN")]
pub struct Shan {
    pub unk1: u32, // some sort of angle
    pub tpcb_count: u32,
    pub unk3: u32, // 0 or 1?
    #[ssbhwrite(align_after = 128)]
    pub name: NameStr,

    // linear interpolation between tpcbs?
    #[br(seek_before = SeekFrom::Start(128))]
    #[br(count = tpcb_count)]
    pub tpcb_starting_frames: Vec<u32>,

    #[br(count = tpcb_count)]
    pub tpcbs: Vec<Ptr32<Tpcb>>,
}

impl Shan {
    /// Tries to read the data from `reader`.
    /// The entire file is buffered for performance.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = Cursor::new(std::fs::read(path)?);
        file.read_le().map_err(Into::into)
    }

    /// Tries to read the data from `reader`.
    /// For best performance when opening from a file, use [Shan::from_file] instead.
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        reader.read_le().map_err(Into::into)
    }

    /// Writes to the given `writer`.
    /// For best performance when writing to a file, use [Shan::write_to_file] instead.
    pub fn write<W: std::io::Write + Seek>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        <Self as SsbhWrite>::write(self, writer)
    }

    /// Writes to the given `path`.
    /// The entire file is buffered for performance.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        // Buffer the entire write operation into memory to improve performance.
        // The seeks used to write relative offsets would cause flushes for BufWriter.
        let mut cursor = Cursor::new(Vec::new());
        self.write(&mut cursor)?;

        let mut writer = std::fs::File::create(path)?;
        writer.write_all(cursor.get_mut())
    }
}

#[binread]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
#[br(magic(b"TPCB"))]
pub struct Tpcb {
    #[br(temp)]
    base_offset: PosValue<()>,

    // These offsets are relative to the start of the struct.
    #[br(temp)]
    offset1: u32,
    #[br(temp)]
    offset2: u32,
    #[br(temp)]
    offset3: u32,

    #[br(args(base_offset.pos, offset1, offset2, offset3))]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub inner: TpcbInner,
}

// Create an inner type to only have to hand write the pointer logic.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, BinRead, SsbhWrite, PartialEq, Clone)]
#[br(import(base_offset: u64, offset1: u32, offset2: u32, offset3: u32))]
pub struct TpcbInner {
    pub header: TpcbHeader,

    // TODO: This needs to account for alignment.
    // Subtract the magic size from each offset.
    /// The range `0..grid_cell_count` not including `grid_cell_count`.
    #[br(args(header.grid_cell_count, base_offset - 4, offset1))]
    pub grid_indices: Grid<u16>,

    /// Compressed spherical harmonic coefficients in row-major order for x -> y -> z.
    #[br(args(header.grid_cell_count, base_offset - 4, offset2))]
    pub grid_sh_coefficients: Grid<CompressedShCoefficients>,

    // TODO: This value isn't always present.
    // TODO: Some sort of location information?
    // TODO: Is this a fixed position for the probes?
    // TODO: Test using a small probe count and different colors.
    // Only used for stage and not chara lighting?
    #[br(args(header.grid_cell_count, base_offset - 4, offset3))]
    pub grid_unk_values: Grid<[f32; 3]>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, BinRead, SsbhWrite, PartialEq, Clone)]
pub struct TpcbHeader {
    pub unk1_1: u16, // TODO: always 1?
    // TODO: Some sort of flags?
    // Related to the unk values?
    // unk1_2, unk values
    // 0000011, 3 (some)
    // 0100011, 35 (none)
    // 1000011, 67 (none)
    // 1010011, 83 (none)
    pub unk1_2: u16,
    pub grid_cell_count_xyz: [u32; 3], // TODO: This can be (0,0,0)?
    // TODO: Setting spacing values to 0 produces all nan coefficients?
    /// `grid_dimensions_xyz / (grid_cell_count_xyz - 1)`
    pub grid_spacing_xyz: [f32; 3],
    /// `grid_range_max_xyz - grid_range_min_xyz`
    pub grid_dimensions_xyz: [f32; 3],
    pub grid_range_min_xyz: [f32; 3],
    pub grid_range_max_xyz: [f32; 3],
    pub unk4: u32, // always 12?
    pub unk5: f32, // affects the grid_sh_coefficients?
    pub unk6: f32, // affects the grid_sh_coefficients?
    /// The product of the counts in `grid_cell_count_xyz`
    pub grid_cell_count: u32,
}

// Values are stored in row major order?
// values[z][y][x]?

// TODO: Provide methods to access the element at a particular x,y,z coordinate?
// ex: tpcb.get_value1(1,2,0).unwrap()
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, SsbhWrite, Clone, PartialEq)]
pub struct Grid<T: BinRead<Args = ()> + SsbhWrite>(pub Option<Vec<T>>);

impl<T: BinRead<Args = ()> + SsbhWrite> BinRead for Grid<T> {
    type Args = (u32, u64, u32);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binrw::ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        // TODO: Named args?
        // Calculate the offset from the start of the TPCB.
        let (count, base_offset, offset) = args;
        let abs_offset = base_offset + offset as u64;

        // Null offsets?
        if offset > 0 {
            let saved_pos = reader.stream_position()?;

            reader.seek(SeekFrom::Start(abs_offset))?;
            let value = binrw::helpers::count(count as usize)(reader, options, ())?;

            reader.seek(SeekFrom::Start(saved_pos))?;
            Ok(Self(Some(value)))
        } else {
            Ok(Self(None))
        }
    }
}

/// Spherical harmonic coefficients for the first two bands.
/// The L0 band has a single coefficient for the constant term.
/// The L1 band has three coefficients for the linear terms.
/// Each coefficient is compressed into a single byte using a linear mapping.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, BinRead, SsbhWrite, Clone, PartialEq)]
pub struct CompressedShCoefficients {
    // TODO: Create types instead of u32.
    // TODO: Expose the coefficient conversion as methods?
    pub r: [u8; 4],
    pub g: [u8; 4],
    pub b: [u8; 4],
}

// TODO: Find a way to derive this.
impl SsbhWrite for Tpcb {
    fn ssbh_write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        data_ptr: &mut u64,
    ) -> std::io::Result<()> {
        // Ensure the next pointer won't point inside this struct.
        let current_pos = writer.stream_position()?;
        if *data_ptr < current_pos + self.size_in_bytes() {
            *data_ptr = current_pos + self.size_in_bytes();
        }
        // Write all the fields.
        writer.write_all(b"TPCB")?;
        // TODO: Is there some kind of alignment for these pointers?
        let offset1 = 96u32; // "header" size including magic?
        let offset2 = offset1 + self.inner.header.grid_cell_count * 2;
        let offset3 = if self.inner.grid_unk_values.0.is_some() {
            offset2 + self.inner.header.grid_cell_count * 12
        } else {
            0
        };

        offset1.ssbh_write(writer, data_ptr)?;
        offset2.ssbh_write(writer, data_ptr)?;
        offset3.ssbh_write(writer, data_ptr)?;

        self.inner.ssbh_write(writer, data_ptr)?;
        Ok(())
    }

    fn alignment_in_bytes() -> u64 {
        16
    }
}

// TODO: Derive proper traits for this like SsbhString.
#[derive(BinRead, SsbhWrite, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "String", into = "String"))]
pub struct NameStr {
    length: u32,
    #[br(count = length)]
    bytes: Vec<u8>,
}

impl From<String> for NameStr {
    fn from(s: String) -> Self {
        let bytes: Vec<u8> = s.as_bytes().into();
        Self {
            length: bytes.len() as u32,
            bytes,
        }
    }
}

impl From<NameStr> for String {
    fn from(n: NameStr) -> Self {
        n.to_string_lossy()
    }
}

impl NameStr {
    /// Converts the underlying buffer to a [String].
    /// See [String::from_utf8_lossy](std::string::String::from_utf8_lossy).
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.bytes).to_string()
    }
}

// TODO: Eventually this should just be the serialize/deserialize implementation.
impl Debug for NameStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.to_string_lossy())
    }
}

use binrw::{prelude::*, VecArgs};
use serde::Serialize;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, SeekFrom};

#[derive(Debug, Serialize)]
pub struct Grid<T: BinRead<Args = ()>>(pub Option<Vec<T>>);

impl<T: BinRead<Args = ()>> BinRead for Grid<T> {
    type Args = (u32, u32, u32);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binrw::ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        // TODO: Named args?
        // Calculate the offset from the start of the TPCB.
        let (count, base_offset, offset) = args;
        let abs_offset = base_offset + offset;

        // Null offsets?
        if offset > 0 {
            let saved_pos = reader.stream_position()?;

            reader.seek(SeekFrom::Start(abs_offset as u64))?;
            let value = <Vec<T>>::read_options(
                reader,
                options,
                VecArgs::<()> {
                    count: count as usize,
                    inner: (),
                },
            )?;

            reader.seek(SeekFrom::Start(saved_pos))?;
            Ok(Self(Some(value)))
        } else {
            Ok(Self(None))
        }
    }
}

// RGB use some sort of compressed representation?
#[derive(Debug, BinRead, BinWrite, Serialize)]
pub struct UnkValue {
    pub r: u32,
    pub g: u32,
    pub b: u32,
}

// Spherical harmonics?
#[derive(Debug, BinRead, Serialize)]
#[br(magic(b"TPCB"))]
#[br(import(base_offset: u32))]
pub struct Tpcb {
    #[serde(skip)]
    pub offset1: u32,
    #[serde(skip)]
    pub offset2: u32,
    #[serde(skip)]
    pub offset3: u32,

    pub unk1_1: u16,
    pub unk1_2: u16,
    pub grid_dimensions: (u32, u32, u32), // TODO: This can be (0,0,0)?

    // Setting all values to 0 produces nan for cbuf11 19,20,21
    // Also affects the intensities calculated from values2?
    pub unks2: [(f32, f32, f32); 4], // angles in degrees?

    // TODO: Is this bit count and min/max for each component?
    pub unk4: u32, // always 12?
    pub unk5: f32, // affects the calculated intensities from values2?
    pub unk6: f32, // affects the calculated intensities from values2?

    pub grid_cell_count: u32, // product of grid_dimensions?

    // Values are stored in row major order?
    // values[z][y][x]?

    // TODO: Provide methods to access the element at a particular x,y,z coordinate?
    // ex: tpcb.get_value1(1,2,0).unwrap()

    // TODO: Find a cleaner way of handling pointers.
    /// Grid cell indices in row-major order.
    #[br(args(grid_cell_count, base_offset, offset1))]
    pub grid_indices: Grid<u16>,

    /// Compressed spherical harmonic coefficients in row-major order.
    #[br(args(grid_cell_count, base_offset, offset2))]
    pub grid_sh_coefficients: Grid<UnkValue>,

    // This value isn't always present.
    // TODO: Some sort of location information?
    #[br(args(grid_cell_count, base_offset, offset3))]
    pub grid_values3: Grid<(f32, f32, f32)>,
}

// TODO: Find a better way to handle args.
// TODO: 16 byte alignment?
#[derive(Debug, Serialize)]
pub struct TpcbPtr(pub Tpcb);

impl BinRead for TpcbPtr {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binrw::ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let offset = u32::read_options(reader, options, ())?;
        let pos_after_read = reader.stream_position()?;

        reader.seek(SeekFrom::Start(offset as u64))?;
        let value = Tpcb::read_options(reader, options, (offset,))?;

        reader.seek(SeekFrom::Start(pos_after_read))?;
        Ok(Self(value))
    }
}

#[derive(BinRead, BinWrite)]
pub struct NameStr {
    length: u32,
    #[br(count = length)]
    bytes: Vec<u8>,
}

impl Serialize for NameStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string_lossy())
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

// Spherical Harmonic ANimation (SHAN)?
#[derive(Debug, BinRead, Serialize)]
#[br(magic(b"SHAN"))]
pub struct Shan {
    pub unk1: u32, // some sort of angle
    pub tpcb_count: u32,
    pub unk3: u32, // 0 or 1?
    pub name: NameStr,

    // The previous space is allocated for the name string.
    // TODO: Check if this is the starting frame index for the tpcbs starting from index 1
    #[br(seek_before = SeekFrom::Start(132))]
    #[br(count = if tpcb_count > 1 { tpcb_count - 1 } else { 0 })]
    pub unks4: Vec<u32>,

    #[br(count = tpcb_count)]
    pub tpcbs: Vec<TpcbPtr>,
}

pub fn read_shan_file(file_name: &str) -> Shan {
    let mut file = BufReader::new(File::open(file_name).unwrap());
    let data: Shan = file.read_le().unwrap();
    data
}
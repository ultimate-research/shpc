# shpc - WIP
A library for reading and writing shpcanim files in Rust. This library is still highly experimental.

SHPC files contain ambient lighting sampled at points in a 3D grid. Lighting data is encoded as spherical harmonic coefficients. Spherical harmonics provide a highly efficient way to encode and evaluate irradiance maps for ambient diffuse lighting. Smash Ultimate uses coefficients for the constant L0 band and linear L1 band compressed into a total of 4 one byte values for the red, green, and blue channels. This is a very crude approximation but avoids the ringing artifacts present with higher order approximations that utilize more coefficients. 

## shpc_json
A program for converting .shpcanim and .shpc files to and from JSON.

## Building
`cargo build --release`

To include this library in a Rust project, add the following line to `cargo.toml`.  
`shpc = { git = "https://github.com/ultimate-research/shpc" }`
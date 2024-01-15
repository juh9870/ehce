// TODO: This is a fallback code, remove once https://github.com/bevyengine/bevy/pull/10392 is merged
use bevy::math::UVec3;
use bevy::prelude::{Color, Image};
use bevy::render::render_resource::{TextureDimension, TextureFormat};
use bevy::render::texture::TextureFormatPixelInfo;
use thiserror::Error;

pub trait ImageExt {
    fn pixel_data_offset(&self, coords: UVec3) -> Option<usize>;
    fn pixel_bytes(&self, coords: UVec3) -> Option<&[u8]>;
    fn pixel_bytes_mut(&mut self, coords: UVec3) -> Option<&mut [u8]>;
    fn get_color_at_1d(&self, x: u32) -> Result<Color, TextureAccessError>;
    fn get_color_at(&self, x: u32, y: u32) -> Result<Color, TextureAccessError>;
    fn get_color_at_3d(&self, x: u32, y: u32, z: u32) -> Result<Color, TextureAccessError>;
    fn set_color_at_1d(&mut self, x: u32, color: Color) -> Result<(), TextureAccessError>;
    fn set_color_at(&mut self, x: u32, y: u32, color: Color) -> Result<(), TextureAccessError>;
    fn set_color_at_3d(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        color: Color,
    ) -> Result<(), TextureAccessError>;
    fn get_color_at_internal(&self, coords: UVec3) -> Result<Color, TextureAccessError>;
    fn set_color_at_internal(
        &mut self,
        coords: UVec3,
        color: Color,
    ) -> Result<(), TextureAccessError>;
}

impl ImageExt for Image {
    /// Compute the byte offset where the data of a specific pixel is stored
    ///
    /// Returns None if the provided coordinates are out of bounds.
    ///
    /// For 2D textures, Z is ignored. For 1D textures, Y and Z are ignored.
    #[inline(always)]
    fn pixel_data_offset(&self, coords: UVec3) -> Option<usize> {
        let width = self.texture_descriptor.size.width;
        let height = self.texture_descriptor.size.height;
        let depth = self.texture_descriptor.size.depth_or_array_layers;

        let pixel_size = self.texture_descriptor.format.pixel_size();
        let pixel_offset = match self.texture_descriptor.dimension {
            TextureDimension::D3 => {
                if coords.x > width || coords.y > height || coords.z > depth {
                    return None;
                }
                coords.z * height * width + coords.y * width + coords.x
            }
            TextureDimension::D2 => {
                if coords.x > width || coords.y > height {
                    return None;
                }
                coords.y * width + coords.x
            }
            TextureDimension::D1 => {
                if coords.x > width {
                    return None;
                }
                coords.x
            }
        };

        Some(pixel_offset as usize * pixel_size)
    }

    /// Get a reference to the data bytes where a specific pixel's value is stored
    #[inline(always)]
    fn pixel_bytes(&self, coords: UVec3) -> Option<&[u8]> {
        let len = self.texture_descriptor.format.pixel_size();
        self.pixel_data_offset(coords)
            .map(|start| &self.data[start..(start + len)])
    }

    /// Get a mutable reference to the data bytes where a specific pixel's value is stored
    #[inline(always)]
    fn pixel_bytes_mut(&mut self, coords: UVec3) -> Option<&mut [u8]> {
        let len = self.texture_descriptor.format.pixel_size();
        self.pixel_data_offset(coords)
            .map(|start| &mut self.data[start..(start + len)])
    }

    /// Read the color of a specific pixel (1D texture).
    ///
    /// See [`get_color_at`] for more details.
    #[inline(always)]
    fn get_color_at_1d(&self, x: u32) -> Result<Color, TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D1 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.get_color_at_internal(UVec3::new(x, 0, 0))
    }

    /// Read the color of a specific pixel (2D texture).
    ///
    /// This function will find the raw byte data of a specific pixel and
    /// decode it into a user-friendly [`Color`] struct for you.
    ///
    /// Supports many of the common [`TextureFormat`]s:
    ///  - RGBA/BGRA 8-bit unsigned integer, both sRGB and Linear
    ///  - 16-bit and 32-bit unsigned integer
    ///  - 32-bit float
    ///
    /// Be careful: as the data is converted to [`Color`] (which uses `f32` internally),
    /// there may be issues with precision when using non-float [`TextureFormat`]s.
    /// If you read a value you previously wrote using `set_color_at`, it will not match.
    /// If you are working with a 32-bit integer [`TextureFormat`], the value will be
    /// inaccurate (as `f32` does not have enough bits to represent it exactly).
    ///
    /// Single channel (R) formats are assumed to represent greyscale, so the value
    /// will be copied to all three RGB channels in the resulting [`Color`].
    ///
    /// Other [`TextureFormat`]s are unsupported, such as:
    ///  - block-compressed formats
    ///  - non-byte-aligned formats like 10-bit
    ///  - 16-bit float formats
    ///  - signed integer formats
    #[inline(always)]
    fn get_color_at(&self, x: u32, y: u32) -> Result<Color, TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D2 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.get_color_at_internal(UVec3::new(x, y, 0))
    }

    /// Read the color of a specific pixel (3D texture).
    ///
    /// See [`get_color_at`] for more details.
    #[inline(always)]
    fn get_color_at_3d(&self, x: u32, y: u32, z: u32) -> Result<Color, TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D3 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.get_color_at_internal(UVec3::new(x, y, z))
    }

    /// Change the color of a specific pixel (1D texture).
    ///
    /// See [`set_color_at`] for more details.
    #[inline(always)]
    fn set_color_at_1d(&mut self, x: u32, color: Color) -> Result<(), TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D1 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.set_color_at_internal(UVec3::new(x, 0, 0), color)
    }

    /// Change the color of a specific pixel (2D texture).
    ///
    /// This function will find the raw byte data of a specific pixel and
    /// change it according to a [`Color`] you provide. The [`Color`] struct
    /// will be encoded into the [`Image`]'s [`TextureFormat`].
    ///
    /// Supports many of the common [`TextureFormat`]s:
    ///  - RGBA/BGRA 8-bit unsigned integer, both sRGB and Linear
    ///  - 16-bit and 32-bit unsigned integer (with possibly-limited precision, as [`Color`] uses `f32`)
    ///  - 32-bit float
    ///
    /// Be careful: writing to non-float [`TextureFormat`]s is lossy! The data has to be converted,
    /// so if you read it back using `get_color_at`, the `Color` you get will not equal the value
    /// you used when writing it using this function.
    ///
    /// For R and RG formats, only the respective values from the linear RGB [`Color`] will be used.
    ///
    /// Other [`TextureFormat`]s are unsupported, such as:
    ///  - block-compressed formats
    ///  - non-byte-aligned formats like 10-bit
    ///  - 16-bit float formats
    ///  - signed integer formats
    #[inline(always)]
    fn set_color_at(&mut self, x: u32, y: u32, color: Color) -> Result<(), TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D2 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.set_color_at_internal(UVec3::new(x, y, 0), color)
    }

    /// Change the color of a specific pixel (3D texture).
    ///
    /// See [`set_color_at`] for more details.
    #[inline(always)]
    fn set_color_at_3d(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        color: Color,
    ) -> Result<(), TextureAccessError> {
        if self.texture_descriptor.dimension != TextureDimension::D3 {
            return Err(TextureAccessError::WrongDimension);
        }
        self.set_color_at_internal(UVec3::new(x, y, z), color)
    }

    #[inline(always)]
    fn get_color_at_internal(&self, coords: UVec3) -> Result<Color, TextureAccessError> {
        let Some(bytes) = self.pixel_bytes(coords) else {
            return Err(TextureAccessError::OutOfBounds {
                x: coords.x,
                y: coords.y,
                z: coords.z,
            });
        };

        match self.texture_descriptor.format {
            TextureFormat::Rgba8UnormSrgb => Ok(Color::rgba(
                bytes[0] as f32 / u8::MAX as f32,
                bytes[1] as f32 / u8::MAX as f32,
                bytes[2] as f32 / u8::MAX as f32,
                bytes[3] as f32 / u8::MAX as f32,
            )),
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Uint => Ok(Color::rgba_linear(
                bytes[0] as f32 / u8::MAX as f32,
                bytes[1] as f32 / u8::MAX as f32,
                bytes[2] as f32 / u8::MAX as f32,
                bytes[3] as f32 / u8::MAX as f32,
            )),
            TextureFormat::Bgra8UnormSrgb => Ok(Color::rgba(
                bytes[2] as f32 / u8::MAX as f32,
                bytes[1] as f32 / u8::MAX as f32,
                bytes[0] as f32 / u8::MAX as f32,
                bytes[3] as f32 / u8::MAX as f32,
            )),
            TextureFormat::Bgra8Unorm => Ok(Color::rgba_linear(
                bytes[2] as f32 / u8::MAX as f32,
                bytes[1] as f32 / u8::MAX as f32,
                bytes[0] as f32 / u8::MAX as f32,
                bytes[3] as f32 / u8::MAX as f32,
            )),
            TextureFormat::Rgba32Float => Ok(Color::rgba_linear(
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            )),
            TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => {
                let (r, g, b, a) = (
                    u16::from_le_bytes([bytes[0], bytes[1]]),
                    u16::from_le_bytes([bytes[2], bytes[3]]),
                    u16::from_le_bytes([bytes[4], bytes[5]]),
                    u16::from_le_bytes([bytes[6], bytes[7]]),
                );
                Ok(Color::rgba_linear(
                    // going via f64 to avoid rounding errors with large numbers and division
                    (r as f64 / u16::MAX as f64) as f32,
                    (g as f64 / u16::MAX as f64) as f32,
                    (b as f64 / u16::MAX as f64) as f32,
                    (a as f64 / u16::MAX as f64) as f32,
                ))
            }
            TextureFormat::Rgba32Uint => {
                let (r, g, b, a) = (
                    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                    u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                    u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
                );
                Ok(Color::rgba_linear(
                    // going via f64 to avoid rounding errors with large numbers and division
                    (r as f64 / u32::MAX as f64) as f32,
                    (g as f64 / u32::MAX as f64) as f32,
                    (b as f64 / u32::MAX as f64) as f32,
                    (a as f64 / u32::MAX as f64) as f32,
                ))
            }
            // assume R-only texture format means grayscale (linear)
            // copy value to all of RGB in Color
            TextureFormat::R8Unorm | TextureFormat::R8Uint => {
                let x = bytes[0] as f32 / u8::MAX as f32;
                Ok(Color::rgba_linear(x, x, x, 1.0))
            }
            TextureFormat::R16Unorm | TextureFormat::R16Uint => {
                let x = u16::from_le_bytes([bytes[0], bytes[1]]);
                // going via f64 to avoid rounding errors with large numbers and division
                let x = (x as f64 / u16::MAX as f64) as f32;
                Ok(Color::rgba_linear(x, x, x, 1.0))
            }
            TextureFormat::R32Uint => {
                let x = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                // going via f64 to avoid rounding errors with large numbers and division
                let x = (x as f64 / u32::MAX as f64) as f32;
                Ok(Color::rgba_linear(x, x, x, 1.0))
            }
            TextureFormat::R32Float => {
                let x = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                Ok(Color::rgba_linear(x, x, x, 1.0))
            }
            TextureFormat::Rg8Unorm | TextureFormat::Rg8Uint => {
                let r = bytes[0] as f32 / u8::MAX as f32;
                let g = bytes[1] as f32 / u8::MAX as f32;
                Ok(Color::rgba_linear(r, g, 0.0, 1.0))
            }
            TextureFormat::Rg16Unorm | TextureFormat::Rg16Uint => {
                let r = u16::from_le_bytes([bytes[0], bytes[1]]);
                let g = u16::from_le_bytes([bytes[2], bytes[3]]);
                // going via f64 to avoid rounding errors with large numbers and division
                let r = (r as f64 / u16::MAX as f64) as f32;
                let g = (g as f64 / u16::MAX as f64) as f32;
                Ok(Color::rgba_linear(r, g, 0.0, 1.0))
            }
            TextureFormat::Rg32Uint => {
                let r = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let g = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                // going via f64 to avoid rounding errors with large numbers and division
                let r = (r as f64 / u32::MAX as f64) as f32;
                let g = (g as f64 / u32::MAX as f64) as f32;
                Ok(Color::rgba_linear(r, g, 0.0, 1.0))
            }
            TextureFormat::Rg32Float => {
                let r = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let g = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                Ok(Color::rgba_linear(r, g, 0.0, 1.0))
            }
            _ => Err(TextureAccessError::UnsupportedTextureFormat(
                self.texture_descriptor.format,
            )),
        }
    }

    #[inline(always)]
    fn set_color_at_internal(
        &mut self,
        coords: UVec3,
        color: Color,
    ) -> Result<(), TextureAccessError> {
        let format = self.texture_descriptor.format;

        let Some(bytes) = self.pixel_bytes_mut(coords) else {
            return Err(TextureAccessError::OutOfBounds {
                x: coords.x,
                y: coords.y,
                z: coords.z,
            });
        };

        match format {
            TextureFormat::Rgba8UnormSrgb => {
                let [r, g, b, a] = color.as_rgba_f32();
                bytes[0] = (r * u8::MAX as f32) as u8;
                bytes[1] = (g * u8::MAX as f32) as u8;
                bytes[2] = (b * u8::MAX as f32) as u8;
                bytes[3] = (a * u8::MAX as f32) as u8;
            }
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8Uint => {
                let [r, g, b, a] = color.as_linear_rgba_f32();
                bytes[0] = (r * u8::MAX as f32) as u8;
                bytes[1] = (g * u8::MAX as f32) as u8;
                bytes[2] = (b * u8::MAX as f32) as u8;
                bytes[3] = (a * u8::MAX as f32) as u8;
            }
            TextureFormat::Bgra8UnormSrgb => {
                let [r, g, b, a] = color.as_rgba_f32();
                bytes[0] = (b * u8::MAX as f32) as u8;
                bytes[1] = (g * u8::MAX as f32) as u8;
                bytes[2] = (r * u8::MAX as f32) as u8;
                bytes[3] = (a * u8::MAX as f32) as u8;
            }
            TextureFormat::Bgra8Unorm => {
                let [r, g, b, a] = color.as_linear_rgba_f32();
                bytes[0] = (b * u8::MAX as f32) as u8;
                bytes[1] = (g * u8::MAX as f32) as u8;
                bytes[2] = (r * u8::MAX as f32) as u8;
                bytes[3] = (a * u8::MAX as f32) as u8;
            }
            TextureFormat::Rgba32Float => {
                let [r, g, b, a] = color.as_linear_rgba_f32();
                bytes[0..4].copy_from_slice(&f32::to_le_bytes(r));
                bytes[4..8].copy_from_slice(&f32::to_le_bytes(g));
                bytes[8..12].copy_from_slice(&f32::to_le_bytes(b));
                bytes[12..16].copy_from_slice(&f32::to_le_bytes(a));
            }
            TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => {
                let [r, g, b, a] = color.as_linear_rgba_f32();
                let [r, g, b, a] = [
                    (r * u16::MAX as f32) as u16,
                    (g * u16::MAX as f32) as u16,
                    (b * u16::MAX as f32) as u16,
                    (a * u16::MAX as f32) as u16,
                ];
                bytes[0..2].copy_from_slice(&u16::to_le_bytes(r));
                bytes[2..4].copy_from_slice(&u16::to_le_bytes(g));
                bytes[4..6].copy_from_slice(&u16::to_le_bytes(b));
                bytes[6..8].copy_from_slice(&u16::to_le_bytes(a));
            }
            TextureFormat::Rgba32Uint => {
                let [r, g, b, a] = color.as_linear_rgba_f32();
                let [r, g, b, a] = [
                    (r * u32::MAX as f32) as u32,
                    (g * u32::MAX as f32) as u32,
                    (b * u32::MAX as f32) as u32,
                    (a * u32::MAX as f32) as u32,
                ];
                bytes[0..4].copy_from_slice(&u32::to_le_bytes(r));
                bytes[4..8].copy_from_slice(&u32::to_le_bytes(g));
                bytes[8..12].copy_from_slice(&u32::to_le_bytes(b));
                bytes[12..16].copy_from_slice(&u32::to_le_bytes(a));
            }
            TextureFormat::R8Unorm | TextureFormat::R8Uint => {
                // TODO: this should probably be changed to do
                // a proper conversion into greyscale
                let [r, _, _, _] = color.as_linear_rgba_f32();
                bytes[0] = (r * u8::MAX as f32) as u8;
            }
            TextureFormat::R16Unorm | TextureFormat::R16Uint => {
                // TODO: this should probably be changed to do
                // a proper conversion into greyscale
                let [r, _, _, _] = color.as_linear_rgba_f32();
                let r = (r * u16::MAX as f32) as u16;
                bytes[0..2].copy_from_slice(&u16::to_le_bytes(r));
            }
            TextureFormat::R32Uint => {
                // TODO: this should probably be changed to do
                // a proper conversion into greyscale
                let [r, _, _, _] = color.as_linear_rgba_f32();
                // go via f64 to avoid imprecision
                let r = (r as f64 * u32::MAX as f64) as u32;
                bytes[0..4].copy_from_slice(&u32::to_le_bytes(r));
            }
            TextureFormat::R32Float => {
                // TODO: this should probably be changed to do
                // a proper conversion into greyscale
                let [r, _, _, _] = color.as_linear_rgba_f32();
                bytes[0..4].copy_from_slice(&f32::to_le_bytes(r));
            }
            TextureFormat::Rg8Unorm | TextureFormat::Rg8Uint => {
                let [r, g, _, _] = color.as_linear_rgba_f32();
                bytes[0] = (r * u8::MAX as f32) as u8;
                bytes[1] = (g * u8::MAX as f32) as u8;
            }
            TextureFormat::Rg16Unorm | TextureFormat::Rg16Uint => {
                let [r, g, _, _] = color.as_linear_rgba_f32();
                let r = (r * u16::MAX as f32) as u16;
                let g = (g * u16::MAX as f32) as u16;
                bytes[0..2].copy_from_slice(&u16::to_le_bytes(r));
                bytes[2..4].copy_from_slice(&u16::to_le_bytes(g));
            }
            TextureFormat::Rg32Uint => {
                let [r, g, _, _] = color.as_linear_rgba_f32();
                // go via f64 to avoid imprecision
                let r = (r as f64 * u32::MAX as f64) as u32;
                let g = (g as f64 * u32::MAX as f64) as u32;
                bytes[0..4].copy_from_slice(&u32::to_le_bytes(r));
                bytes[4..8].copy_from_slice(&u32::to_le_bytes(g));
            }
            TextureFormat::Rg32Float => {
                let [r, g, _, _] = color.as_linear_rgba_f32();
                bytes[0..4].copy_from_slice(&f32::to_le_bytes(r));
                bytes[4..8].copy_from_slice(&f32::to_le_bytes(g));
            }
            _ => {
                return Err(TextureAccessError::UnsupportedTextureFormat(
                    self.texture_descriptor.format,
                ));
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum TextureAccessError {
    #[error("out of bounds (x: {x}, y: {y}, z: {z})")]
    OutOfBounds { x: u32, y: u32, z: u32 },
    #[error("unsupported texture format: {0:?}")]
    UnsupportedTextureFormat(TextureFormat),
    #[error("attempt to access texture with different dimension")]
    WrongDimension,
}

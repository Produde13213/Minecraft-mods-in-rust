//! Sodium-style compact 20-byte terrain vertex encoding.
//! Clean-room and independent of Minecraft types; adapters map baked quad data to InputVertex.

use crate::Vertex;

pub const STRIDE: usize = 20;
pub const POSITION_MAX: u32 = 1 << 20;
const TEXTURE_MAX: f32 = 1 << 15;
const MODEL_ORIGIN: f32 = 8.0;
const MODEL_RANGE: f32 = 32.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputVertex {
    pub position: [f32; 3],
    pub color: [u8; 4],
    pub uv: [f32; 2],
    /// Packed light: block in bits 0..7, sky in bits 16..23.
    pub packed_light: u32,
    pub ao: f32,
}

impl From<Vertex> for InputVertex {
    fn from(v: Vertex) -> Self {
        let packed_light = u32::from(v.light[0]) | (u32::from(v.light[1]) << 16);
        Self { position: v.position, color: v.color, uv: v.uv, packed_light, ao: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    NonFinitePosition,
    NonFiniteTextureCoordinate,
    NonFiniteAmbientOcclusion,
    InvalidAmbientOcclusion,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for EncodeError {}

/// Encodes one quad into 80 bytes, little-endian. Material and section IDs use 8 bits each.
pub fn encode_quad(vertices: [InputVertex; 4], material_bits: u32, section_id: u32)
    -> Result<[u8; STRIDE * 4], EncodeError>
{
    let mut center = [0.0f32; 2];
    for v in &vertices {
        if v.position.iter().any(|x| !x.is_finite()) { return Err(EncodeError::NonFinitePosition); }
        if v.uv.iter().any(|x| !x.is_finite()) { return Err(EncodeError::NonFiniteTextureCoordinate); }
        if !v.ao.is_finite() { return Err(EncodeError::NonFiniteAmbientOcclusion); }
        if !(0.0..=1.0).contains(&v.ao) { return Err(EncodeError::InvalidAmbientOcclusion); }
        center[0] += v.uv[0] * 0.25;
        center[1] += v.uv[1] * 0.25;
    }

    let mut out = [0u8; STRIDE * 4];
    for (i, v) in vertices.iter().enumerate() {
        let p = v.position.map(quantize_position);
        let uv = [encode_texture(center[0], v.uv[0]), encode_texture(center[1], v.uv[1])];
        let light = encode_light(v.packed_light);
        let color = multiply_rgb(v.color, v.ao);
        let words = [
            pack_position_hi(p[0], p[1], p[2]),
            pack_position_lo(p[0], p[1], p[2]),
            u32::from_le_bytes(color),
            u32::from(uv[0]) | (u32::from(uv[1]) << 16),
            u32::from(light) | ((material_bits & 0xff) << 16) | ((section_id & 0xff) << 24),
        ];
        for (word_index, word) in words.into_iter().enumerate() {
            let start = i * STRIDE + word_index * 4;
            out[start..start + 4].copy_from_slice(&word.to_le_bytes());
        }
    }
    Ok(out)
}

fn quantize_position(position: f32) -> u32 {
    ((((MODEL_ORIGIN + position) / MODEL_RANGE) * POSITION_MAX as f32) as i64 as u32) & 0x000f_ffff
}
fn pack_position_hi(x: u32, y: u32, z: u32) -> u32 {
    ((x >> 10) & 0x3ff) | (((y >> 10) & 0x3ff) << 10) | (((z >> 10) & 0x3ff) << 20)
}
fn pack_position_lo(x: u32, y: u32, z: u32) -> u32 {
    (x & 0x3ff) | ((y & 0x3ff) << 10) | ((z & 0x3ff) << 20)
}
fn encode_texture(center: f32, coordinate: f32) -> u16 {
    let bias: i32 = if coordinate < center { 1 } else { -1 };
    let quantized = (coordinate * TEXTURE_MAX).round() as i32 + bias;
    let sign_bit = if bias < 0 { 1u16 } else { 0u16 };
    ((quantized as u16) & 0x7fff) | (sign_bit << 15)
}
fn encode_light(packed: u32) -> u16 {
    let sky = (((packed >> 16) & 0xff) as i32 + 8).clamp(8, 248) as u16;
    let block = ((packed & 0xff) as i32 + 8).clamp(8, 248) as u16;
    block | (sky << 8)
}
fn multiply_rgb(mut color: [u8; 4], ao: f32) -> [u8; 4] {
    for channel in &mut color[..3] {
        *channel = ((*channel as f32 * ao).round() as i32).clamp(0, 255) as u8;
    }
    color
}

#[cfg(test)]
mod tests {
    use super::*;
    fn vertex(position: [f32; 3], uv: [f32; 2]) -> InputVertex {
        InputVertex { position, color: [200, 100, 50, 255], uv, packed_light: 0, ao: 0.5 }
    }
    fn word(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
    #[test]
    fn emits_four_twenty_byte_vertices() {
        let encoded = encode_quad([
            vertex([0.0,0.0,0.0],[0.0,0.0]), vertex([1.0,0.0,0.0],[1.0,0.0]),
            vertex([1.0,1.0,0.0],[1.0,1.0]), vertex([0.0,1.0,0.0],[0.0,1.0]),
        ], 0x12, 0x34).unwrap();
        assert_eq!(encoded.len(), 80);
        assert_eq!(word(&encoded, 8), u32::from_le_bytes([100,50,25,255]));
        assert_eq!(word(&encoded, 16) >> 16, 0x3412);
    }
    #[test]
    fn lightmap_channels_are_clamped_and_biased() {
        assert_eq!(encode_light(0), 0x0808);
        assert_eq!(encode_light(0x00ff_00ff), 0xf8f8);
        assert_eq!(encode_light(0x0080_0080), 0x8888);
    }
    #[test]
    fn position_axes_are_split_into_twenty_bits() {
        let x = quantize_position(0.0);
        let y = quantize_position(16.0);
        let z = quantize_position(-8.0);
        assert!(x < POSITION_MAX && y < POSITION_MAX && z < POSITION_MAX);
        assert_eq!(pack_position_hi(x,y,z) & 0x3ff, (x >> 10) & 0x3ff);
        assert_eq!(pack_position_lo(x,y,z) & 0x3ff, x & 0x3ff);
    }
    #[test]
    fn rejects_invalid_inputs() {
        let mut q = [vertex([0.0;3],[0.0;2]);4];
        q[2].position[1] = f32::NAN;
        assert_eq!(encode_quad(q,0,0), Err(EncodeError::NonFinitePosition));
        q[2].position[1] = 0.0;
        q[0].ao = 1.5;
        assert_eq!(encode_quad(q,0,0), Err(EncodeError::InvalidAmbientOcclusion));
    }
}

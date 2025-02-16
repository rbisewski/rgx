#![deny(clippy::all, clippy::use_self)]
#![allow(clippy::cast_lossless)]

use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use raw_window_handle::RawWindowHandle;

use crate::math;
use crate::math::{Point2, Vector2};

///////////////////////////////////////////////////////////////////////////
// Rgba8
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const WHITE: Self = Self {
        r: 0xff,
        g: 0xff,
        b: 0xff,
        a: 0xff,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0xff,
    };
    pub const RED: Self = Self {
        r: 0xff,
        g: 0,
        b: 0,
        a: 0xff,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 0xff,
        b: 0,
        a: 0xff,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 0xff,
        a: 0xff,
    };

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl fmt::Display for Rgba8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl From<Rgba> for Rgba8 {
    fn from(rgba: Rgba) -> Self {
        Self {
            r: (rgba.r * 255.0).round() as u8,
            g: (rgba.g * 255.0).round() as u8,
            b: (rgba.b * 255.0).round() as u8,
            a: (rgba.a * 255.0).round() as u8,
        }
    }
}

impl From<u32> for Rgba8 {
    fn from(rgba: u32) -> Self {
        unsafe { std::mem::transmute(rgba) }
    }
}

impl FromStr for Rgba8 {
    type Err = std::num::ParseIntError;

    /// Parse a color code of the form '#ffffff' into an
    /// instance of 'Rgba8'. The alpha is always 0xff.
    fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
        let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;
        let a: u8 = 0xff;

        Ok(Rgba8 { r, g, b, a })
    }
}

/// A BGRA color, used when dealing with framebuffers.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Bgra8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Bgra8 {
    pub const fn new(b: u8, g: u8, r: u8, a: u8) -> Self {
        Bgra8 { b, g, r, a }
    }
}

impl From<Rgba8> for Bgra8 {
    fn from(rgba: Rgba8) -> Self {
        Self {
            b: rgba.b,
            g: rgba.g,
            r: rgba.r,
            a: rgba.a,
        }
    }
}

impl Into<Rgba8> for Bgra8 {
    fn into(self) -> Rgba8 {
        Rgba8 {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Rect
///////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Rect<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}

impl<T> Rect<T> {
    pub const fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn zero() -> Self
    where
        T: math::Zero,
    {
        Self {
            x1: T::zero(),
            x2: T::zero(),
            y1: T::zero(),
            y2: T::zero(),
        }
    }

    pub fn origin(w: T, h: T) -> Self
    where
        T: math::Zero,
    {
        Self::new(T::zero(), T::zero(), w, h)
    }

    pub fn map<F, S>(self, f: F) -> Rect<S>
    where
        F: Fn(T) -> S,
    {
        Rect {
            x1: f(self.x1),
            x2: f(self.x2),
            y1: f(self.y1),
            y2: f(self.y2),
        }
    }

    pub fn scale(&self, x: T, y: T) -> Self
    where
        T: std::ops::Mul<Output = T> + Copy,
    {
        Self {
            x1: self.x1,
            y1: self.y1,
            x2: self.x2 * x,
            y2: self.y2 * y,
        }
    }

    pub fn translate(&self, x: T, y: T) -> Self
    where
        T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + Copy,
    {
        Self {
            x1: x,
            y1: y,
            x2: x + (self.x2 - self.x1),
            y2: y + (self.y2 - self.y1),
        }
    }

    pub fn is_empty(&self) -> bool
    where
        T: PartialEq,
    {
        self.x1 == self.x2 && self.y1 == self.y2
    }

    pub fn is_zero(&self) -> bool
    where
        T: math::Zero,
    {
        self.x1.is_zero() && self.x2.is_zero() && self.y1.is_zero() && self.y2.is_zero()
    }

    pub fn width(&self) -> T
    where
        T: Copy + PartialOrd + std::ops::Sub<Output = T> + std::ops::Neg<Output = T> + math::Zero,
    {
        let w = self.x2 - self.x1;
        if w < T::zero() {
            -w
        } else {
            w
        }
    }

    /// Return the height of the rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    ///
    /// let r = Rect::origin(-6, -6);
    /// assert_eq!(r.height(), 6);
    /// ```
    pub fn height(&self) -> T
    where
        T: Copy + PartialOrd + std::ops::Sub<Output = T> + std::ops::Neg<Output = T> + math::Zero,
    {
        let h = self.y2 - self.y1;
        if h < T::zero() {
            -h
        } else {
            h
        }
    }

    /// Return the minimum point of a rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    /// use rgx::math::Point2;
    ///
    /// let r = Rect::new(0, 0, 1, -1);
    /// assert_eq!(r.min(), Point2::new(0, -1));
    /// ```
    pub fn min(&self) -> Point2<T>
    where
        T: Ord + Copy,
    {
        Point2::new(T::min(self.x1, self.x2), T::min(self.y1, self.y2))
    }

    /// Return the maximum point of a rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    /// use rgx::math::Point2;
    ///
    /// let r = Rect::origin(-1, 1);
    /// assert_eq!(r.max(), Point2::new(0, 1));
    /// ```
    pub fn max(&self) -> Point2<T>
    where
        T: Ord + Copy,
    {
        Point2::new(T::max(self.x1, self.x2), T::max(self.y1, self.y2))
    }

    /// Return the center of the rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    /// use rgx::math::Point2;
    ///
    /// let r = Rect::origin(8, 8);
    /// assert_eq!(r.center(), Point2::new(4, 4));
    ///
    /// let r = Rect::new(0, 0, -8, -8);
    /// assert_eq!(r.center(), Point2::new(-4, -4));
    /// ```
    pub fn center(&self) -> Point2<T>
    where
        T: std::ops::Div<Output = T>
            + Copy
            + Ord
            + From<i16>
            + PartialOrd
            + math::Zero
            + std::ops::Neg<Output = T>
            + std::ops::Sub<Output = T>,
    {
        let r = self.normalized();
        Point2::new(r.x1 + r.width() / 2.into(), r.y1 + r.height() / 2.into())
    }

    pub fn radius(&self) -> T
    where
        T: std::ops::Div<Output = T>
            + Copy
            + From<i16>
            + PartialOrd
            + math::Zero
            + std::ops::Neg<Output = T>
            + std::ops::Sub<Output = T>,
    {
        let w = self.width();
        let h = self.height();

        if w > h {
            w / 2.into()
        } else {
            h / 2.into()
        }
    }

    pub fn contains(&self, p: Point2<T>) -> bool
    where
        T: PartialOrd,
    {
        p.x >= self.x1 && p.x <= self.x2 && p.y >= self.y1 && p.y <= self.y2
    }

    pub fn intersects(&self, other: Rect<T>) -> bool
    where
        T: PartialOrd,
    {
        self.y2 > other.y1 && self.y1 < other.y2 && self.x1 < other.x2 && self.x2 > other.x1
    }

    /// Return the normalized rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    ///
    /// let r = Rect::new(3, 3, 1, 1).normalized();
    /// assert_eq!(r, Rect::new(1, 1, 3, 3));
    ///
    /// let r = Rect::new(-1, -1, 1, 1).normalized();
    /// assert_eq!(r, Rect::new(-1, -1, 1, 1));
    /// ```
    pub fn normalized(&self) -> Rect<T>
    where
        T: Ord + Copy,
    {
        Rect::new(
            T::min(self.x1, self.x2),
            T::min(self.y1, self.y2),
            T::max(self.x1, self.x2),
            T::max(self.y1, self.y2),
        )
    }

    /// Return a rectangle clamped to the given bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::core::Rect;
    ///
    /// let bounds = Rect::new(0, 0, 3, 3);
    ///
    /// let r = Rect::new(1, 1, 6, 6);
    /// assert_eq!(r.clamped(bounds), Rect::new(1, 1, 3, 3));
    ///
    /// let r = Rect::new(1, 1, 2, 2);
    /// assert_eq!(r.clamped(bounds), Rect::new(1, 1, 2, 2));
    ///
    /// let r = Rect::new(-1, -1, 3, 3);
    /// assert_eq!(r.clamped(bounds), Rect::new(0, 0, 3, 3));
    /// ```
    pub fn clamped(&self, bounds: Rect<T>) -> Self
    where
        T: Ord + Copy,
    {
        Rect::new(
            T::max(self.x1, bounds.x1),
            T::max(self.y1, bounds.y1),
            T::min(self.x2, bounds.x2),
            T::min(self.y2, bounds.y2),
        )
    }
}

impl<T> std::ops::Add<Vector2<T>> for Rect<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, vec: Vector2<T>) -> Self {
        Self {
            x1: self.x1 + vec.x,
            y1: self.y1 + vec.y,
            x2: self.x2 + vec.x,
            y2: self.y2 + vec.y,
        }
    }
}

impl<T> std::ops::AddAssign<Vector2<T>> for Rect<T>
where
    T: std::ops::AddAssign<T> + Copy,
{
    fn add_assign(&mut self, vec: Vector2<T>) {
        self.x1 += vec.x;
        self.y1 += vec.y;
        self.x2 += vec.x;
        self.y2 += vec.y;
    }
}

impl<T> std::ops::Sub<Vector2<T>> for Rect<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, vec: Vector2<T>) -> Self {
        Self {
            x1: self.x1 - vec.x,
            y1: self.y1 - vec.y,
            x2: self.x2 - vec.x,
            y2: self.y2 - vec.y,
        }
    }
}

impl<T> std::ops::SubAssign<Vector2<T>> for Rect<T>
where
    T: std::ops::SubAssign<T> + Copy,
{
    fn sub_assign(&mut self, vec: Vector2<T>) {
        self.x1 -= vec.x;
        self.y1 -= vec.y;
        self.x2 -= vec.x;
        self.y2 -= vec.y;
    }
}

impl<T> std::ops::Mul<T> for Rect<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, s: T) -> Self {
        Self {
            x1: self.x1 * s,
            y1: self.y1 * s,
            x2: self.x2 * s,
            y2: self.y2 * s,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Draw
///////////////////////////////////////////////////////////////////////////////

pub trait Draw {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass);
}

///////////////////////////////////////////////////////////////////////////////
/// Rgba
///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl From<Rgba8> for Rgba {
    fn from(rgba8: Rgba8) -> Self {
        Self {
            r: (rgba8.r as f32 / 255.0),
            g: (rgba8.g as f32 / 255.0),
            b: (rgba8.b as f32 / 255.0),
            a: (rgba8.a as f32 / 255.0),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Shaders
///////////////////////////////////////////////////////////////////////////////

pub struct Shader {
    module: wgpu::ShaderModule,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl ShaderStage {
    fn to_wgpu(&self) -> wgpu::ShaderStage {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStage::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStage::FRAGMENT,
            ShaderStage::Compute => wgpu::ShaderStage::COMPUTE,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Canvas
///////////////////////////////////////////////////////////////////////////////

pub trait Canvas {
    fn clear(&self, color: Rgba, device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn fill(&self, buf: &[u8], device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn transfer(
        &self,
        buf: &[u8],
        w: u32,
        h: u32,
        tw: u32,
        th: u32,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    );
    fn blit(&self, from: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder);
}

///////////////////////////////////////////////////////////////////////////////
/// BindingGroup
///////////////////////////////////////////////////////////////////////////////

/// A group of bindings.
pub struct BindingGroup {
    wgpu: wgpu::BindGroup,
    set_index: u32,
}

impl BindingGroup {
    fn new(set_index: u32, wgpu: wgpu::BindGroup) -> Self {
        Self { set_index, wgpu }
    }
}

/// The layout of a 'BindingGroup'.
pub struct BindingGroupLayout {
    wgpu: wgpu::BindGroupLayout,
    size: usize,
    set_index: u32,
}

impl BindingGroupLayout {
    fn new(set_index: u32, layout: wgpu::BindGroupLayout, size: usize) -> Self {
        Self {
            wgpu: layout,
            size,
            set_index,
        }
    }
}

/// A trait representing a resource that can be bound.
pub trait Bind {
    fn binding(&self, index: u32) -> wgpu::Binding;
}

///////////////////////////////////////////////////////////////////////////////
/// Uniforms
///////////////////////////////////////////////////////////////////////////////

/// A uniform buffer that can be bound in a 'BindingGroup'.
pub struct UniformBuffer {
    wgpu: wgpu::Buffer,
    size: usize,
    count: usize,
}

impl Bind for UniformBuffer {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Buffer {
                buffer: &self.wgpu,
                range: 0..(self.size as wgpu::BufferAddress),
            },
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Framebuffer
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct Framebuffer {
    pub texture: Texture,
}

impl Framebuffer {
    pub fn size(&self) -> usize {
        (self.texture.w * self.texture.h) as usize
    }

    pub fn width(&self) -> u32 {
        self.texture.w
    }

    pub fn height(&self) -> u32 {
        self.texture.h
    }
}

impl Bind for Framebuffer {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.texture.view),
        }
    }
}

impl Canvas for Framebuffer {
    fn clear(&self, color: Rgba, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(
            &self.texture,
            Bgra8::from(Rgba8::from(color)),
            device,
            encoder,
        );
    }

    fn fill(&self, buf: &[u8], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self.texture, buf, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[u8],
        w: u32,
        h: u32,
        tw: u32,
        th: u32,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self.texture, buf, w, h, tw, th, device, encoder);
    }

    fn blit(&self, from: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        Texture::blit(&self.texture, from, dst, encoder);
    }
}

impl TextureView for Framebuffer {
    fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Texturing
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct Texture {
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,
    extent: wgpu::Extent3d,

    pub w: u32,
    pub h: u32,
}

impl Texture {
    pub fn rect(&self) -> Rect<f32> {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: self.w as f32,
            y2: self.h as f32,
        }
    }

    fn clear<T>(
        texture: &Texture,
        color: T,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Into<Rgba8> + Clone,
    {
        let mut texels: Vec<T> = Vec::with_capacity(texture.w as usize * texture.h as usize);
        texels.resize(texture.w as usize * texture.h as usize, color);

        let (head, body, tail) = unsafe { texels.align_to::<u8>() };
        assert!(head.is_empty());
        assert!(tail.is_empty());

        Self::fill(texture, body, device, encoder);
    }

    fn fill(
        texture: &Texture,
        texels: &[u8],
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        assert_eq!(
            texels.len() as u32,
            texture.w * texture.h * 4,
            "fatal: incorrect length for texel buffer"
        );

        let buf = device
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&texels);

        Self::copy(
            &texture.wgpu,
            texture.w,
            texture.h,
            texture.extent,
            &buf,
            encoder,
        );
    }

    fn transfer(
        texture: &Texture,
        texels: &[u8],
        width: u32,
        height: u32,
        transfer_w: u32,
        transfer_h: u32,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        assert_eq!(
            (texels.len() as u32 / 4),
            width * height,
            "fatal: incorrect length for texel buffer"
        );
        assert!(
            transfer_w * transfer_h <= texture.w * texture.h,
            "fatal: transfer size must be <= texture size"
        );

        let buf = device
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&texels);

        let extent = wgpu::Extent3d {
            width: transfer_w,
            height: transfer_h,
            depth: 1,
        };
        Self::copy(&texture.wgpu, width, height, extent, &buf, encoder);
    }

    fn blit(&self, src: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        assert_eq!(
            src.width(),
            dst.width(),
            "source and destination rectangles must be of the same size"
        );
        assert_eq!(
            src.height(),
            dst.height(),
            "source and destination rectangles must be of the same size"
        );

        encoder.copy_texture_to_texture(
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: src.x1,
                    y: src.y1,
                    z: 0.0,
                },
            },
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: dst.x1,
                    y: dst.y1,
                    z: 0.0,
                },
            },
            wgpu::Extent3d {
                width: src.width() as u32,
                height: src.height() as u32,
                depth: 1,
            },
        );
    }

    fn copy(
        texture: &wgpu::Texture,
        w: u32,
        h: u32,
        extent: wgpu::Extent3d,
        buffer: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                offset: 0,
                row_pitch: 4 * w,
                image_height: h,
            },
            wgpu::TextureCopyView {
                texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            extent,
        );
    }
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

impl Canvas for Texture {
    fn fill(&self, buf: &[u8], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self, buf, device, encoder);
    }

    fn clear(&self, color: Rgba, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self, color, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[u8],
        w: u32,
        h: u32,
        tw: u32,
        th: u32,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self, buf, w, h, tw, th, device, encoder);
    }

    fn blit(&self, src: Rect<f32>, dst: Rect<f32>, encoder: &mut wgpu::CommandEncoder) {
        Texture::blit(&self, src, dst, encoder);
    }
}

impl From<Framebuffer> for Texture {
    fn from(fb: Framebuffer) -> Self {
        fb.texture
    }
}

pub struct Sampler {
    wgpu: wgpu::Sampler,
}

impl Bind for Sampler {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Sampler(&self.wgpu),
        }
    }
}

#[derive(Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    fn to_wgpu(&self) -> wgpu::FilterMode {
        match self {
            Filter::Nearest => wgpu::FilterMode::Nearest,
            Filter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Vertex/Index Buffers
///////////////////////////////////////////////////////////////////////////////

pub struct VertexBuffer {
    pub size: u32,
    wgpu: wgpu::Buffer,
}

impl Draw for VertexBuffer {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass) {
        // TODO: If we attempt to draw more vertices than exist in the buffer, because
        // 'size' was guessed wrong, we get a wgpu error. We should somehow try to
        // get the pipeline layout to know here if the buffer we're trying to draw
        // is the right size. Another option is to create buffers from the pipeline,
        // so that we can check at creation time whether the data passed in matches
        // the format.
        pass.set_binding(binding, &[]);
        pass.draw_buffer(&self);
    }
}

pub struct IndexBuffer {
    wgpu: wgpu::Buffer,
}

#[derive(Clone, Copy)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    UByte4,
}

impl VertexFormat {
    // TODO: Use `const fn`
    fn bytesize(self) -> usize {
        match self {
            VertexFormat::Float => 4,
            VertexFormat::Float2 => 8,
            VertexFormat::Float3 => 12,
            VertexFormat::Float4 => 16,
            VertexFormat::UByte4 => 4,
        }
    }
    // TODO: Use `const fn`
    fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            VertexFormat::Float => wgpu::VertexFormat::Float,
            VertexFormat::Float2 => wgpu::VertexFormat::Float2,
            VertexFormat::Float3 => wgpu::VertexFormat::Float3,
            VertexFormat::Float4 => wgpu::VertexFormat::Float4,
            VertexFormat::UByte4 => wgpu::VertexFormat::Uchar4Norm,
        }
    }
}

/// Describes a 'VertexBuffer' layout.
#[derive(Default)]
pub struct VertexLayout {
    wgpu_attrs: Vec<wgpu::VertexAttributeDescriptor>,
    size: usize,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.wgpu_attrs.push(wgpu::VertexAttributeDescriptor {
                shader_location: vl.wgpu_attrs.len() as u32,
                offset: vl.size as wgpu::BufferAddress,
                format: vf.to_wgpu(),
            });
            vl.size += vf.bytesize();
        }
        vl
    }

    fn to_wgpu(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: self.size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: self.wgpu_attrs.as_slice(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline Bindings
///////////////////////////////////////////////////////////////////////////////

/// A binding type.
pub enum BindingType {
    UniformBuffer,
    UniformBufferDynamic,
    Sampler,
    SampledTexture,
}

impl BindingType {
    fn to_wgpu(&self) -> wgpu::BindingType {
        match self {
            BindingType::UniformBuffer => wgpu::BindingType::UniformBuffer { dynamic: false },
            BindingType::UniformBufferDynamic => wgpu::BindingType::UniformBuffer { dynamic: true },
            BindingType::SampledTexture => wgpu::BindingType::SampledTexture {
                multisampled: false,
                dimension: wgpu::TextureViewDimension::D2,
            },
            BindingType::Sampler => wgpu::BindingType::Sampler,
        }
    }
}

pub struct Binding {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline
///////////////////////////////////////////////////////////////////////////////

pub struct Pipeline {
    wgpu: wgpu::RenderPipeline,

    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

impl<'a> AbstractPipeline<'a> for Pipeline {
    type PrepareContext = ();
    type Uniforms = ();

    fn description() -> PipelineDescription<'a> {
        PipelineDescription {
            vertex_layout: &[],
            pipeline_layout: &[],
            vertex_shader: &[],
            fragment_shader: &[],
        }
    }

    fn setup(pipeline: Self, _dev: &Device, _w: u32, _h: u32) -> Self {
        pipeline
    }

    fn apply(&self, pass: &mut Pass) {
        pass.wgpu.set_pipeline(&self.wgpu);
    }

    fn width(&self) -> u32 {
        unimplemented!()
    }

    fn height(&self) -> u32 {
        unimplemented!()
    }

    fn resize(&mut self, _w: u32, _h: u32) {}

    fn prepare(&'a self, _unused: ()) -> Option<(&'a UniformBuffer, Vec<()>)> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blending {
    src_factor: BlendFactor,
    dst_factor: BlendFactor,
    operation: BlendOp,
}

impl Blending {
    pub fn new(src_factor: BlendFactor, dst_factor: BlendFactor, operation: BlendOp) -> Self {
        Blending {
            src_factor,
            dst_factor,
            operation,
        }
    }

    pub fn constant() -> Self {
        Blending {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOp::Add,
        }
    }

    fn to_wgpu(&self) -> (wgpu::BlendFactor, wgpu::BlendFactor, wgpu::BlendOperation) {
        (
            self.src_factor.to_wgpu(),
            self.dst_factor.to_wgpu(),
            self.operation.to_wgpu(),
        )
    }
}

impl Default for Blending {
    fn default() -> Self {
        Blending {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOp::Add,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    One,
    Zero,
    SrcAlpha,
    OneMinusSrcAlpha,
}

impl BlendFactor {
    fn to_wgpu(&self) -> wgpu::BlendFactor {
        match self {
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendOp {
    Add,
}

impl BlendOp {
    fn to_wgpu(&self) -> wgpu::BlendOperation {
        match self {
            BlendOp::Add => wgpu::BlendOperation::Add,
        }
    }
}

pub struct Set<'a>(pub &'a [Binding]);

pub struct PipelineLayout {
    pub sets: Vec<BindingGroupLayout>,
}

pub trait AbstractPipeline<'a> {
    type PrepareContext;
    type Uniforms: Copy + 'static;

    fn description() -> PipelineDescription<'a>;
    fn setup(pip: Pipeline, dev: &Device, w: u32, h: u32) -> Self;
    fn apply(&self, pass: &mut Pass);
    fn resize(&mut self, w: u32, h: u32);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn prepare(
        &'a self,
        t: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)>;
}

pub struct PipelineDescription<'a> {
    pub vertex_layout: &'a [VertexFormat],
    pub pipeline_layout: &'a [Set<'a>],
    pub vertex_shader: &'static [u8],
    pub fragment_shader: &'static [u8],
}

///////////////////////////////////////////////////////////////////////////////
/// Frame
///////////////////////////////////////////////////////////////////////////////

pub struct Frame {
    encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self { encoder }
    }

    pub fn pass<T: TextureView>(&mut self, op: PassOp, view: &T) -> Pass {
        Pass::begin(&mut self.encoder, &view.texture_view(), op)
    }

    pub fn copy(&mut self, src: &UniformBuffer, dst: &UniformBuffer) {
        self.encoder.copy_buffer_to_buffer(
            &src.wgpu,
            0,
            &dst.wgpu,
            0,
            (src.size * src.count) as wgpu::BufferAddress,
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pass
///////////////////////////////////////////////////////////////////////////////

pub struct Pass<'a> {
    wgpu: wgpu::RenderPass<'a>,
}

impl<'a> Pass<'a> {
    pub fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        op: PassOp,
    ) -> Self {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                load_op: op.to_wgpu(),
                store_op: wgpu::StoreOp::Store,
                clear_color: match op {
                    PassOp::Clear(color) => color.to_wgpu(),
                    PassOp::Load() => Rgba::TRANSPARENT.to_wgpu(),
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: None,
        });
        Pass { wgpu: pass }
    }
    pub fn set_pipeline<T>(&mut self, pipeline: &T)
    where
        T: AbstractPipeline<'a>,
    {
        pipeline.apply(self);
    }
    pub fn set_binding(&mut self, group: &BindingGroup, offsets: &[u64]) {
        self.wgpu
            .set_bind_group(group.set_index, &group.wgpu, offsets);
    }
    pub fn set_index_buffer(&mut self, index_buf: &IndexBuffer) {
        self.wgpu.set_index_buffer(&index_buf.wgpu, 0)
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &VertexBuffer) {
        self.wgpu.set_vertex_buffers(0, &[(&vertex_buf.wgpu, 0)])
    }
    pub fn draw<T: Draw>(&mut self, drawable: &T, binding: &BindingGroup) {
        drawable.draw(binding, self);
    }
    pub fn draw_buffer(&mut self, buf: &VertexBuffer) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(0..buf.size, 0..1);
    }
    pub fn draw_buffer_range(&mut self, buf: &VertexBuffer, range: Range<u32>) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(range, 0..1);
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

pub enum PassOp {
    Clear(Rgba),
    Load(),
}

impl PassOp {
    fn to_wgpu(&self) -> wgpu::LoadOp {
        match self {
            PassOp::Clear(_) => wgpu::LoadOp::Clear,
            PassOp::Load() => wgpu::LoadOp::Load,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// SwapChain & TextureView
///////////////////////////////////////////////////////////////////////////////

pub trait TextureView {
    fn texture_view(&self) -> &wgpu::TextureView;
}

pub struct SwapChainTexture<'a>(wgpu::SwapChainOutput<'a>);

impl TextureView for SwapChainTexture<'_> {
    fn texture_view(&self) -> &wgpu::TextureView {
        &self.0.view
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Vsync,
    NoVsync,
}

impl PresentMode {
    fn to_wgpu(&self) -> wgpu::PresentMode {
        match self {
            PresentMode::Vsync => wgpu::PresentMode::Vsync,
            PresentMode::NoVsync => wgpu::PresentMode::NoVsync,
        }
    }
}

impl Default for PresentMode {
    fn default() -> Self {
        PresentMode::Vsync
    }
}

/// A handle to a swap chain.
///
/// A `SwapChain` represents the image or series of images that will be presented to a [`Renderer`].
/// A `SwapChain` may be created with [`Renderer::swap_chain`].
pub struct SwapChain {
    pub width: u32,
    pub height: u32,

    wgpu: wgpu::SwapChain,
}

impl SwapChain {
    /// Convenience method to retrieve `(width, height)`
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the next texture to be presented by the swapchain for drawing.
    ///
    /// When the [`SwapChainTexture`] returned by this method is dropped, the
    /// swapchain will present the texture to the associated [`Renderer`].
    pub fn next(&mut self) -> SwapChainTexture {
        SwapChainTexture(self.wgpu.get_next_texture())
    }

    fn descriptor(width: u32, height: u32, mode: PresentMode) -> wgpu::SwapChainDescriptor {
        wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            present_mode: mode.to_wgpu(),
            width,
            height,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Renderer
///////////////////////////////////////////////////////////////////////////////

pub struct Renderer {
    pub device: Device,
}

impl Renderer {
    pub fn new(window: RawWindowHandle) -> Self {
        Self {
            device: Device::new(window),
        }
    }

    pub fn swap_chain(&self, w: u32, h: u32, mode: PresentMode) -> SwapChain {
        SwapChain {
            wgpu: self.device.create_swap_chain(w, h, mode),
            width: w,
            height: h,
        }
    }

    pub fn texture(&self, w: u32, h: u32) -> Texture {
        self.device.create_texture(w, h)
    }

    pub fn framebuffer(&self, w: u32, h: u32) -> Framebuffer {
        self.device.create_framebuffer(w, h)
    }

    pub fn vertex_buffer<T>(&self, verts: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_buffer(verts)
    }

    pub fn uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_uniform_buffer(buf)
    }

    pub fn binding_group(&self, layout: &BindingGroupLayout, binds: &[&dyn Bind]) -> BindingGroup {
        self.device.create_binding_group(layout, binds)
    }

    pub fn sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        self.device.create_sampler(min_filter, mag_filter)
    }

    pub fn pipeline<T>(&self, w: u32, h: u32, blending: Blending) -> T
    where
        T: AbstractPipeline<'static>,
    {
        let desc = T::description();
        let pip_layout = self.device.create_pipeline_layout(desc.pipeline_layout);
        let vertex_layout = VertexLayout::from(desc.vertex_layout);
        let vs =
            self.device
                .create_shader("vertex shader", desc.vertex_shader, ShaderStage::Vertex);
        let fs = self.device.create_shader(
            "fragment shader",
            desc.fragment_shader,
            ShaderStage::Fragment,
        );

        T::setup(
            self.device
                .create_pipeline(pip_layout, vertex_layout, blending, &vs, &fs),
            &self.device,
            w,
            h,
        )
    }

    pub fn read<F>(&mut self, fb: &Framebuffer, f: F)
    where
        F: 'static + FnOnce(&[u8]),
    {
        let mut encoder = self.device.create_command_encoder();

        let bytesize = 4 * fb.size();
        let dst = self.device.device.create_buffer(&wgpu::BufferDescriptor {
            size: bytesize as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &fb.texture.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            wgpu::BufferCopyView {
                buffer: &dst,
                offset: 0,
                // TODO: Must be a multiple of 256
                row_pitch: 4 * fb.texture.w,
                image_height: fb.texture.h,
            },
            fb.texture.extent,
        );
        self.device.submit(&[encoder.finish()]);

        let mut buffer: Vec<u8> = Vec::with_capacity(bytesize);

        dst.map_read_async(
            0,
            bytesize as u64,
            move |result: wgpu::BufferMapAsyncResult<&[u8]>| match result {
                Ok(ref mapping) => {
                    buffer.extend_from_slice(mapping.data);
                    if buffer.len() == bytesize {
                        f(unsafe { std::mem::transmute(buffer.as_slice()) });
                    }
                }
                Err(ref err) => panic!("{:?}", err),
            },
        );
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn update_pipeline<'a, T>(&mut self, pip: &'a T, p: T::PrepareContext, f: &mut Frame)
    where
        T: AbstractPipeline<'a>,
    {
        if let Some((buf, unifs)) = pip.prepare(p) {
            self.device
                .update_uniform_buffer::<T::Uniforms>(unifs.as_slice(), buf, &mut f.encoder);
        }
    }

    pub fn frame(&mut self) -> Frame {
        let encoder = self.device.create_command_encoder();
        Frame::new(encoder)
    }

    pub fn submit(&mut self, frame: Frame) {
        self.device.submit(&[frame.encoder.finish()]);
    }

    pub fn prepare(&mut self, commands: &[Op]) {
        let mut encoder = self.device.create_command_encoder();
        for c in commands.iter() {
            c.encode(&mut self.device, &mut encoder);
        }
        self.device.submit(&[encoder.finish()]);
    }
}

pub enum Op<'a> {
    Clear(&'a dyn Canvas, Rgba),
    Fill(&'a dyn Canvas, &'a [u8]),
    Transfer(&'a dyn Canvas, &'a [u8], u32, u32, u32, u32),
    Blit(&'a dyn Canvas, Rect<f32>, Rect<f32>),
}

impl<'a> Op<'a> {
    fn encode(&self, dev: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        match *self {
            Op::Clear(f, color) => {
                f.clear(color, dev, encoder);
            }
            Op::Fill(f, buf) => {
                f.fill(buf, dev, encoder);
            }
            Op::Transfer(f, buf, w, h, tw, th) => {
                f.transfer(buf, w, h, tw, th, dev, encoder);
            }
            Op::Blit(f, src, dst) => {
                f.blit(src, dst, encoder);
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Device
///////////////////////////////////////////////////////////////////////////////

pub struct Device {
    device: wgpu::Device,
    surface: wgpu::Surface,
}

impl Device {
    pub fn new(window: RawWindowHandle) -> Self {
        let instance = wgpu::Instance::new();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
        });
        let surface = instance.create_surface(window);

        Self {
            device: adapter.request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            }),
            surface,
        }
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }

    pub fn create_swap_chain(&self, w: u32, h: u32, mode: PresentMode) -> wgpu::SwapChain {
        let desc = SwapChain::descriptor(w, h, mode);
        self.device.create_swap_chain(&self.surface, &desc)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_binding_group_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
    }

    pub fn create_shader(&self, _name: &str, source: &[u8], _stage: ShaderStage) -> Shader {
        let buf = std::io::Cursor::new(source);
        let spv = wgpu::read_spirv(buf).unwrap();

        Shader {
            module: self.device.create_shader_module(spv.as_slice()),
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }

    pub fn create_texture(&self, w: u32, h: u32) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();

        Texture {
            wgpu: texture,
            view: texture_view,
            extent: texture_extent,
            w,
            h,
        }
    }

    pub fn create_framebuffer(&self, w: u32, h: u32) -> Framebuffer {
        let extent = wgpu::Extent3d {
            width: w,
            height: h,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = texture.create_default_view();

        Framebuffer {
            texture: Texture {
                wgpu: texture,
                view,
                extent,
                w,
                h,
            },
        }
    }

    pub fn create_binding_group(
        &self,
        layout: &BindingGroupLayout,
        binds: &[&dyn Bind],
    ) -> BindingGroup {
        assert_eq!(
            binds.len(),
            layout.size,
            "layout slot count does not match bindings"
        );

        let mut bindings = Vec::new();

        for (i, b) in binds.iter().enumerate() {
            bindings.push(b.binding(i as u32));
        }

        BindingGroup::new(
            layout.set_index,
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout.wgpu,
                bindings: bindings.as_slice(),
            }),
        )
    }

    pub fn create_buffer<T>(&self, vertices: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self
                .device
                .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
                .fill_from_slice(vertices),
            size: vertices.len() as u32,
        }
    }

    pub fn create_uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            count: buf.len(),
            wgpu: self
                .device
                .create_buffer_mapped::<T>(
                    buf.len(),
                    wgpu::BufferUsage::UNIFORM
                        | wgpu::BufferUsage::COPY_DST
                        | wgpu::BufferUsage::COPY_SRC,
                )
                .fill_from_slice(buf),
        }
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(indices);
        IndexBuffer { wgpu: index_buf }
    }

    pub fn create_sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        Sampler {
            wgpu: self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: mag_filter.to_wgpu(),
                min_filter: min_filter.to_wgpu(),
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare_function: wgpu::CompareFunction::Always,
            }),
        }
    }

    pub fn create_binding_group_layout(&self, index: u32, slots: &[Binding]) -> BindingGroupLayout {
        let mut bindings = Vec::new();

        for s in slots {
            bindings.push(wgpu::BindGroupLayoutBinding {
                binding: bindings.len() as u32,
                visibility: s.stage.to_wgpu(),
                ty: s.binding.to_wgpu(),
            });
        }
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: bindings.as_slice(),
            });
        BindingGroupLayout::new(index, layout, bindings.len())
    }

    pub fn update_uniform_buffer<T: Copy + 'static>(
        &self,
        slice: &[T],
        buf: &UniformBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let src = self
            .device
            .create_buffer_mapped::<T>(
                slice.len(),
                wgpu::BufferUsage::UNIFORM
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::MAP_WRITE,
            )
            .fill_from_slice(slice);

        encoder.copy_buffer_to_buffer(
            &src,
            0,
            &buf.wgpu,
            0,
            (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress,
        );
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn submit(&mut self, cmds: &[wgpu::CommandBuffer]) {
        self.device.get_queue().submit(cmds);
    }

    // PRIVATE API ////////////////////////////////////////////////////////////

    fn create_pipeline(
        &self,
        pipeline_layout: PipelineLayout,
        vertex_layout: VertexLayout,
        blending: Blending,
        vs: &Shader,
        fs: &Shader,
    ) -> Pipeline {
        let vertex_attrs = vertex_layout.to_wgpu();

        let mut sets = Vec::new();
        for s in pipeline_layout.sets.iter() {
            sets.push(&s.wgpu);
        }
        let layout = &self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: sets.as_slice(),
            });

        let (src_factor, dst_factor, operation) = blending.to_wgpu();

        let wgpu = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs.module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs.module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    // TODO: Try Bgra8UnormSrgb
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_attrs],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Pipeline {
            layout: pipeline_layout,
            vertex_layout,
            wgpu,
        }
    }
}

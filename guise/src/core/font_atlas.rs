use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::char;
use core::ops::{BitOr, BitOrAssign, RangeInclusive};

use hashbrown::hash_map::{DefaultHashBuilder, Entry, HashMap};

use crate::convert::{cast_u16, cast_u32, cast_usize};

// TODO(yan): @Portability @Speed @Memory Think about having the user provide
// the font atlas, or adding a mechanism for building the font-atlas in compile
// time or as a build step.

#[cfg(feature = "font_ibm_plex_mono")]
pub static FONT_IBM_PLEX_MONO: &[u8] = include_bytes!("../../assets/IBMPlexMono-Regular.ttf");
#[cfg(feature = "font_ibm_plex_sans_jp")]
pub static FONT_IBM_PLEX_SANS_JP: &[u8] = include_bytes!("../../assets/IBMPlexSansJP-Regular.ttf");
#[cfg(feature = "font_proggy_clean")]
pub static FONT_PROGGY_CLEAN: &[u8] = include_bytes!("../../assets/ProggyClean.ttf");
#[cfg(feature = "font_roboto")]
pub static FONT_ROBOTO: &[u8] = include_bytes!("../../assets/Roboto-Regular.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnicodeRangeFlags(u32);

impl UnicodeRangeFlags {
    pub const BASIC_LATIN: Self = Self(0x01);
    pub const LATIN_1_SUPPLEMENT: Self = Self(0x02);
    pub const LATIN_EXTENDED_A: Self = Self(0x04);
    pub const LATIN_EXTENDED_B: Self = Self(0x08);

    pub const CJK_SYMBOLS_AND_PUNCTUATION: Self = Self(0x10);
    pub const HIRAGANA: Self = Self(0x20);
    pub const KATAKANA: Self = Self(0x40);
    pub const CJK_UNIFIED_IDEOGRAPHS: Self = Self(0x80);

    pub const ALL_LATIN: Self = Self::BASIC_LATIN
        | Self::LATIN_1_SUPPLEMENT
        | Self::LATIN_EXTENDED_A
        | Self::LATIN_EXTENDED_B;

    pub const ALL_JAPANESE: Self = Self::CJK_SYMBOLS_AND_PUNCTUATION
        | Self::HIRAGANA
        | Self::KATAKANA
        | Self::CJK_UNIFIED_IDEOGRAPHS;

    pub const ALL: Self = Self::ALL_LATIN | Self::ALL_JAPANESE;

    const R_BASIC_LATIN: RangeInclusive<u32> = 0x00..=0x7f;
    const R_LATIN_1_SUPPLEMENT: RangeInclusive<u32> = 0x80..=0xff;
    const R_LATIN_EXTENDED_A: RangeInclusive<u32> = 0x0100..=0x017f;
    const R_LATIN_EXTENDED_B: RangeInclusive<u32> = 0x0180..=0x024f;

    const R_CJK_SYMBOLS_AND_PUNCTUATION: RangeInclusive<u32> = 0x3000..=0x303f;
    const R_HIRAGANA: RangeInclusive<u32> = 0x3040..=0x309f;
    const R_KATAKANA: RangeInclusive<u32> = 0x30a0..=0x30ff;
    const R_CJK_UNIFIED_IDEOGRAPHS: RangeInclusive<u32> = 0x4e00..=0x9fff;

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn from_bits_truncate(bits: u32) -> Self {
        Self(Self::ALL.0 & bits)
    }

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn codepoint_count(&self) -> u32 {
        let mut count: u32 = 0;

        if self.0 & Self::BASIC_LATIN.0 != 0 {
            count += 1 + Self::R_BASIC_LATIN.end() - Self::R_BASIC_LATIN.start();
        }

        if self.0 & Self::LATIN_1_SUPPLEMENT.0 != 0 {
            count += 1 + Self::R_LATIN_1_SUPPLEMENT.end() - Self::R_LATIN_1_SUPPLEMENT.start();
        }

        if self.0 & Self::LATIN_EXTENDED_A.0 != 0 {
            count += 1 + Self::R_LATIN_EXTENDED_A.end() - Self::R_LATIN_EXTENDED_A.start();
        }

        if self.0 & Self::LATIN_EXTENDED_B.0 != 0 {
            count += 1 + Self::R_LATIN_EXTENDED_B.end() - Self::R_LATIN_EXTENDED_B.start();
        }

        if self.0 & Self::CJK_SYMBOLS_AND_PUNCTUATION.0 != 0 {
            count += 1 + Self::R_CJK_SYMBOLS_AND_PUNCTUATION.end()
                - Self::R_CJK_SYMBOLS_AND_PUNCTUATION.start();
        }

        if self.0 & Self::HIRAGANA.0 != 0 {
            count += 1 + Self::R_HIRAGANA.end() - Self::R_HIRAGANA.start();
        }

        if self.0 & Self::KATAKANA.0 != 0 {
            count += 1 + Self::R_KATAKANA.end() - Self::R_KATAKANA.start();
        }

        if self.0 & Self::CJK_UNIFIED_IDEOGRAPHS.0 != 0 {
            count +=
                1 + Self::R_CJK_UNIFIED_IDEOGRAPHS.end() - Self::R_CJK_UNIFIED_IDEOGRAPHS.start();
        }

        count
    }

    pub fn codepoint_ranges_iter(&self) -> CodepointRangesIter {
        CodepointRangesIter {
            next: 0,
            flags: *self,
        }
    }
}

impl const BitOr for UnicodeRangeFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for UnicodeRangeFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

pub struct CodepointRangesIter {
    next: usize,
    flags: UnicodeRangeFlags,
}

impl Iterator for CodepointRangesIter {
    type Item = RangeInclusive<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        use UnicodeRangeFlags as F;
        static RANGES: &[(UnicodeRangeFlags, RangeInclusive<u32>)] = &[
            (F::BASIC_LATIN, F::R_BASIC_LATIN),
            (F::LATIN_1_SUPPLEMENT, F::R_LATIN_1_SUPPLEMENT),
            (F::LATIN_EXTENDED_A, F::R_LATIN_EXTENDED_A),
            (F::LATIN_EXTENDED_B, F::R_LATIN_EXTENDED_B),
            (
                F::CJK_SYMBOLS_AND_PUNCTUATION,
                F::R_CJK_SYMBOLS_AND_PUNCTUATION,
            ),
            (F::HIRAGANA, F::R_HIRAGANA),
            (F::KATAKANA, F::R_KATAKANA),
            (F::CJK_UNIFIED_IDEOGRAPHS, F::R_CJK_UNIFIED_IDEOGRAPHS),
        ];

        while self.next < RANGES.len() {
            let index = self.next;
            self.next += 1;

            let (range_flags, range) = RANGES[index].clone();

            if self.flags.intersects(range_flags) {
                return Some(range);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInfo {
    // The cell of the atlas where the glyph is.
    pub grid_x: u16,
    pub grid_y: u16,

    // Glyph advance with in logical pixels, used for layout.
    pub advance_width: f32,

    // Glyph width and height in logical pixels, used for layout.
    pub width: f32,
    pub height: f32,

    // See fontdue::Metrics::xmin and fontdue::Metrics::ymin
    pub xmin: i32,
    pub ymin: i32,

    // Glyph width and height in the atlas, where they are scaled.
    pub atlas_width: f32,
    pub atlas_height: f32,
}

// TODO(yan): Allocate everything in provided allocator.
pub struct FontAtlas<A: Allocator + Clone> {
    font: fontdue::Font,
    font_horizontal_line_metrics: fontdue::LineMetrics,
    max_atlas_glyph_width: u16,
    max_atlas_glyph_height: u16,
    image: Vec<u8>,
    image_width: u16,
    image_height: u16,
    glyph_index_to_info: HashMap<usize, GlyphInfo, DefaultHashBuilder, A>,
    missing_glyph_info: GlyphInfo,
}

impl<A: Allocator + Clone> FontAtlas<A> {
    pub fn new_in<TA: Allocator>(
        font_bytes: &[u8],
        unicode_range_flags: UnicodeRangeFlags,
        font_size: f32,
        font_scale_factor: f32,
        perm_allocator: A,
        temp_allocator: &TA,
    ) -> FontAtlas<A> {
        let min_font_scale = fontdue::FontSettings::default().scale;
        let font_size_scaled = font_size * font_scale_factor;

        let settings = fontdue::FontSettings {
            collection_index: 0,
            // Scale controls the threshold of subdividing a line segment. A
            // very low scale will produce low poly glyph geometry. This will
            // become visible once the font size becomes large enough. In
            // practice, my blind eyes only start seeing degradation when scale
            // is half the font size, so keeping it around font-sized should be
            // ok. We might want to revisit this later.
            scale: f32::max(min_font_scale, font_size_scaled),
        };
        let font = fontdue::Font::from_bytes(font_bytes, settings).unwrap();

        // Keep line metrics in logical pixels (w/o font_scale_factor applied) so
        // that all layout computation is in logical units, but rasterize the
        // atlas scaled for high DPI, if requested.
        let font_horizontal_line_metrics = font.horizontal_line_metrics(font_size).unwrap();

        let codepoint_count = unicode_range_flags.codepoint_count();
        guise_log!("Generating font atlas from {} codepoints", codepoint_count);

        let mut max_atlas_glyph_width: u16 = 0;
        let mut max_atlas_glyph_height: u16 = 0;
        let mut glyph_index_to_rasterized =
            HashMap::with_capacity_in(cast_usize(codepoint_count), temp_allocator);

        for c in unicode_range_flags
            .codepoint_ranges_iter()
            .flatten()
            .filter_map(char::from_u32)
        {
            // 0-th index maps to the font's default character - we want
            // to process it too, so we can render it. Therefore, we do
            // not special-case it.
            let glyph_index = font.lookup_glyph_index(c);

            // Multiple codepoints can map to the same index. We
            // therefore check whether we already processed this one.
            if let Entry::Vacant(vacant_entry) = glyph_index_to_rasterized.entry(glyph_index) {
                // NB: Rasterize with scale factor applied, but also get
                // unscaled metrics for layout in logical pixels.
                //
                // TODO(yan): Font::rasterized_subpixel
                let (metrics, image) = font.rasterize_indexed(glyph_index, font_size_scaled);
                let unscaled_metrics = font.metrics_indexed(glyph_index, font_size);

                let width = cast_u16(metrics.width);
                let height = cast_u16(metrics.height);

                if width > max_atlas_glyph_width {
                    max_atlas_glyph_width = width;
                }
                if height > max_atlas_glyph_height {
                    max_atlas_glyph_height = height;
                }

                vacant_entry.insert((metrics, unscaled_metrics, image));
            }
        }

        // +1, because we are adding an opaque cell at the start of the atlas.
        let atlas_cell_count = cast_u32(glyph_index_to_rasterized.len()) + 1;
        let (atlas_pixel_width, atlas_pixel_height) = find_atlas_image_size(
            atlas_cell_count,
            max_atlas_glyph_width,
            max_atlas_glyph_height,
        );
        let atlas_grid_width = atlas_pixel_width / max_atlas_glyph_width;
        let atlas_grid_height = atlas_cell_count / u32::from(atlas_grid_width) + 1;

        guise_log!(
            "Generating font atlas: {}x{} ({}x{})",
            atlas_pixel_width,
            atlas_pixel_height,
            atlas_grid_width,
            atlas_grid_height,
        );

        let mut atlas_image =
            vec![0; usize::from(atlas_pixel_width) * usize::from(atlas_pixel_height) * 4];

        // Blit glyph-sized maxvalue rectangle at the first position in the atlas.
        //
        // NB: Upcast to usize to prevent overflows in multiplication below,
        // when computing index.
        for y in 0..usize::from(max_atlas_glyph_height) {
            for x in 0..usize::from(max_atlas_glyph_width) {
                let index = usize::from((x + y * usize::from(atlas_pixel_width)) * 4);
                atlas_image[index] = 255;
                atlas_image[index + 1] = 255;
                atlas_image[index + 2] = 255;
                atlas_image[index + 3] = 255;
            }
        }

        let mut glyph_index_to_info =
            HashMap::with_capacity_in(cast_usize(codepoint_count), perm_allocator);

        let mut cell_index = 1;
        for c in unicode_range_flags
            .codepoint_ranges_iter()
            .flatten()
            .filter_map(char::from_u32)
        {
            let glyph_index = font.lookup_glyph_index(c);

            if let Entry::Vacant(vacant_entry) = glyph_index_to_info.entry(glyph_index) {
                let (metrics, unscaled_metrics, image) = &glyph_index_to_rasterized[&glyph_index];

                let grid_x = cell_index % usize::from(atlas_grid_width);
                let grid_y = cell_index / usize::from(atlas_grid_width);

                let pixel_x = grid_x * usize::from(max_atlas_glyph_width);
                let pixel_y = grid_y * usize::from(max_atlas_glyph_height);

                // Blit glyph into font atlas. Fill RGB with white so that we
                // don't bleed. This works, because the rendering backend is
                // expected to multiply this with a color.
                debug_assert!(usize::from(max_atlas_glyph_width) >= metrics.width);
                debug_assert!(usize::from(max_atlas_glyph_height) >= metrics.height);
                for src_pixel_y in 0..metrics.height {
                    for src_pixel_x in 0..metrics.width {
                        let dst_pixel_x = pixel_x + src_pixel_x;
                        let dst_pixel_y = pixel_y + src_pixel_y;

                        let src_index = src_pixel_x + src_pixel_y * metrics.width;
                        let dst_index =
                            (dst_pixel_x + dst_pixel_y * usize::from(atlas_pixel_width)) * 4;

                        atlas_image[dst_index] = 255;
                        atlas_image[dst_index + 1] = 255;
                        atlas_image[dst_index + 2] = 255;
                        atlas_image[dst_index + 3] = image[src_index];
                    }
                }

                vacant_entry.insert(GlyphInfo {
                    grid_x: cast_u16(grid_x),
                    grid_y: cast_u16(grid_y),

                    advance_width: unscaled_metrics.advance_width,

                    width: unscaled_metrics.width as f32,
                    height: unscaled_metrics.height as f32,

                    xmin: unscaled_metrics.xmin,
                    ymin: unscaled_metrics.ymin,

                    atlas_width: metrics.width as f32,
                    atlas_height: metrics.height as f32,
                });

                cell_index += 1;
            }
        }

        // TODO(yan): This needs audit and tuning.
        let missing_glyph_info = {
            let sf = font_scale_factor;

            const ADVANCE_SIZE_RATIO: f32 = 0.8;
            const SIZE_RATIO: f32 = 0.7;

            let horizontal_padding = max_atlas_glyph_width as f32 * (1.0 - SIZE_RATIO) / sf;
            let vertical_padding = max_atlas_glyph_height as f32 * (1.0 - SIZE_RATIO) / sf;

            let advance_width = max_atlas_glyph_width as f32 * ADVANCE_SIZE_RATIO / sf;
            let width = max_atlas_glyph_width as f32 * SIZE_RATIO / sf;
            let height = max_atlas_glyph_height as f32 * SIZE_RATIO / sf;

            let atlas_width = max_atlas_glyph_width as f32 * SIZE_RATIO;
            let atlas_height = max_atlas_glyph_height as f32 * SIZE_RATIO;

            GlyphInfo {
                grid_x: 0,
                grid_y: 0,

                advance_width,

                width,
                height,

                xmin: (horizontal_padding / 2.0) as i32,
                ymin: (vertical_padding / 2.0) as i32,

                atlas_width,
                atlas_height,
            }
        };

        Self {
            font,
            font_horizontal_line_metrics,
            max_atlas_glyph_width,
            max_atlas_glyph_height,
            image: atlas_image,
            image_width: atlas_pixel_width,
            image_height: atlas_pixel_height,
            glyph_index_to_info,
            missing_glyph_info,
        }
    }

    pub fn grid_cell_size(&self) -> (u16, u16) {
        (self.max_atlas_glyph_width, self.max_atlas_glyph_height)
    }

    pub fn image_size(&self) -> (u16, u16) {
        (self.image_width, self.image_height)
    }

    pub fn image_rgba8_unorm(&self) -> &[u8] {
        &self.image
    }

    pub fn line_metrics(&self) -> fontdue::LineMetrics {
        self.font_horizontal_line_metrics
    }

    pub fn glyph_info(&self, c: char) -> GlyphInfo {
        // This has two tiers of failure. If fontdue doesn't recognize the
        // glyph, it returns the index for the missing character. However, there
        // might not be a glyph info for the missing character, and we fabricate
        // one if it doesn't exist.
        let glyph_index = self.font.lookup_glyph_index(c);
        if let Some(glyph_info) = self.glyph_index_to_info.get(&glyph_index) {
            *glyph_info
        } else {
            self.missing_glyph_info()
        }
    }

    pub fn missing_glyph_info(&self) -> GlyphInfo {
        self.missing_glyph_info
    }
}

fn find_atlas_image_size(cell_count: u32, cell_width: u16, cell_height: u16) -> (u16, u16) {
    fn evaluate(
        atlas_width: u16,
        atlas_height: u16,
        cell_count: u32,
        cell_width: u16,
        cell_height: u16,
    ) -> bool {
        let cells_per_row = atlas_width / cell_width;
        if cells_per_row == 0 {
            return false;
        }

        let row_count = cell_count / u32::from(cells_per_row) + 1;
        let required_pixel_height = row_count * u32::from(cell_height);

        required_pixel_height <= u32::from(atlas_height)
    }

    let mut power_of_two_prev: u16 = 1;
    let mut power_of_two: u16 = power_of_two_prev
        .checked_add(1)
        .unwrap()
        .checked_next_power_of_two()
        .unwrap();

    while !evaluate(
        power_of_two,
        power_of_two,
        cell_count,
        cell_width,
        cell_height,
    ) {
        power_of_two_prev = power_of_two;
        power_of_two = power_of_two
            .checked_add(1)
            .unwrap()
            .checked_next_power_of_two()
            .unwrap();
    }

    if evaluate(
        power_of_two,
        power_of_two_prev,
        cell_count,
        cell_width,
        cell_height,
    ) {
        (power_of_two, power_of_two_prev)
    } else {
        (power_of_two, power_of_two)
    }
}

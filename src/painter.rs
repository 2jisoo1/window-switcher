use crate::app::SwitchAppsState;
use crate::utils::{check_error, get_moinitor_rect, is_light_theme, is_win11};

use anyhow::{Context, Result};
use windows::core::w;
use windows::Win32::{
    Foundation::{COLORREF, HWND, POINT, RECT, SIZE},
    Graphics::{
        Gdi::{
            CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateRoundRectRgn,
            CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, FillRect, FillRgn, GetDC,
            ReleaseDC, SelectObject, SetBkMode, SetStretchBltMode, SetTextColor, StretchBlt,
            AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BLENDFUNCTION, CLIP_DEFAULT_PRECIS,
            DEFAULT_CHARSET, DT_CENTER, DT_NOPREFIX, DT_TOP, DT_WORDBREAK, HALFTONE, HBITMAP, HDC,
            HPALETTE, OUT_DEFAULT_PRECIS, SRCCOPY, TRANSPARENT,
        },
        GdiPlus::{
            FillModeAlternate, GdipAddPathArc, GdipClosePathFigure, GdipCreateBitmapFromHBITMAP,
            GdipCreateFromHDC, GdipCreatePath, GdipCreatePen1, GdipDeleteBrush, GdipDeleteGraphics,
            GdipDeletePath, GdipDeletePen, GdipDisposeImage, GdipDrawImageRect, GdipFillPath,
            GdipFillRectangle, GdipGetPenBrushFill, GdipSetInterpolationMode, GdipSetSmoothingMode,
            GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, GpBitmap, GpBrush, GpGraphics,
            GpImage, GpPath, GpPen, InterpolationModeHighQualityBicubic, SmoothingModeAntiAlias,
            Unit,
        },
    },
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            DrawIconEx, GetCursorPos, ShowWindow, UpdateLayeredWindow, DI_NORMAL, SW_HIDE, SW_SHOW,
            ULW_ALPHA,
        },
    },
};

pub const BG_DARK_COLOR: u32 = 0x4c4c4c;
pub const FG_DARK_COLOR: u32 = 0x3b3b3b;
pub const BG_LIGHT_COLOR: u32 = 0xe0e0e0;
pub const FG_LIGHT_COLOR: u32 = 0xf2f2f2;
pub const ALPHA_MASK: u32 = 0xff000000;
pub const ICON_SIZE_BASE: i32 = 64;
pub const WINDOW_BORDER_SIZE_BASE: i32 = 10;
pub const ICON_BORDER_SIZE_BASE: i32 = 4;
pub const ICON_GAP_BASE: i32 = 12;
pub const SCALE_FACTOR: i32 = 6;
pub const LABEL_FONT_SIZE_BASE: i32 = 13;
pub const LABEL_LINE_HEIGHT_BASE: i32 = 16;
pub const LABEL_LINES: i32 = 2;
pub const LABEL_TOP_GAP_BASE: i32 = 2;
pub const TEXT_DARK_COLOR: u32 = 0x202020;
pub const TEXT_LIGHT_COLOR: u32 = 0xf0f0f0;

// GDI Antialiasing Painter
pub struct GdiAAPainter {
    token: usize,
    hwnd: HWND,
    hdc_screen: HDC,
    rounded_corner: bool,
    show: bool,
}

impl GdiAAPainter {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let startup_input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let mut token: usize = 0;
        check_error(|| unsafe { GdiplusStartup(&mut token, &startup_input, std::ptr::null_mut()) })
            .context("Failed to initialize GDI+")?;

        let hdc_screen = unsafe { GetDC(Some(hwnd)) };
        let rounded_corner = is_win11();

        Ok(Self {
            token,
            hwnd,
            hdc_screen,
            rounded_corner,
            show: false,
        })
    }

    pub fn paint(&mut self, state: &SwitchAppsState) {
        let dpi_scale = get_dpi_scale(self.hwnd);
        let icon_size_max = (ICON_SIZE_BASE as f64 * dpi_scale) as i32;
        let border_size = (WINDOW_BORDER_SIZE_BASE as f64 * dpi_scale) as i32;
        let icon_border = (ICON_BORDER_SIZE_BASE as f64 * dpi_scale) as i32;
        let icon_gap = (ICON_GAP_BASE as f64 * dpi_scale) as i32;
        let (label_height, label_font_size) = label_metrics(dpi_scale);

        let Coordinate {
            x,
            y,
            width,
            height,
            icon_size,
            item_size,
            icon_cell,
        } = Coordinate::new(
            state.apps.len() as i32,
            icon_size_max,
            border_size,
            icon_border,
            icon_gap,
            label_height,
        );

        let corner_radius = if self.rounded_corner {
            icon_cell / 4
        } else {
            0
        };

        let hwnd = self.hwnd;
        let hdc_screen = self.hdc_screen;

        let light_theme = is_light_theme();
        let (fg_color, bg_color) = theme_color(light_theme);
        let text_color = if light_theme {
            TEXT_DARK_COLOR
        } else {
            TEXT_LIGHT_COLOR
        };

        unsafe {
            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            let bitmap_mem = CreateCompatibleBitmap(hdc_screen, width, height);
            SelectObject(hdc_mem, bitmap_mem.into());

            let mut graphics = GpGraphics::default();
            let mut graphics_ptr: *mut GpGraphics = &mut graphics;
            GdipCreateFromHDC(hdc_mem, &mut graphics_ptr as _);
            GdipSetSmoothingMode(graphics_ptr, SmoothingModeAntiAlias);
            GdipSetInterpolationMode(graphics_ptr, InterpolationModeHighQualityBicubic);

            let mut bg_pen = GpPen::default();
            let mut bg_pen_ptr: *mut GpPen = &mut bg_pen;
            GdipCreatePen1(ALPHA_MASK | bg_color, 0.0, Unit(0), &mut bg_pen_ptr as _);

            let mut bg_brush = GpBrush::default();
            let mut bg_brush_ptr: *mut GpBrush = &mut bg_brush;
            GdipGetPenBrushFill(bg_pen_ptr, &mut bg_brush_ptr as _);

            if self.rounded_corner {
                draw_round_rect(
                    graphics_ptr,
                    bg_brush_ptr,
                    0.0,
                    0.0,
                    width as f32,
                    height as f32,
                    corner_radius as f32,
                );
            } else {
                GdipFillRectangle(
                    graphics_ptr,
                    bg_brush_ptr,
                    0.0,
                    0.0,
                    width as f32,
                    height as f32,
                );
            }

            let icons_width = item_size * state.apps.len() as i32;
            let icons_height = icon_cell + label_height;
            let bitmap_icons = draw_icons(
                state,
                hdc_screen,
                icon_size,
                icon_border,
                icons_width,
                icons_height,
                corner_radius,
                fg_color,
                bg_color,
                item_size,
                label_font_size,
                text_color,
            );

            let mut bitmap = GpBitmap::default();
            let mut bitmap_ptr: *mut GpBitmap = &mut bitmap as _;
            GdipCreateBitmapFromHBITMAP(bitmap_icons, HPALETTE::default(), &mut bitmap_ptr as _);

            let image_ptr: *mut GpImage = bitmap_ptr as *mut GpImage;
            GdipDrawImageRect(
                graphics_ptr,
                image_ptr,
                border_size as f32,
                border_size as f32,
                icons_width as f32,
                icons_height as f32,
            );

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as _,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as _,
                ..Default::default()
            };
            let _ = UpdateLayeredWindow(
                hwnd,
                Some(hdc_screen),
                Some(&POINT { x, y }),
                Some(&SIZE {
                    cx: width,
                    cy: height,
                }),
                Some(hdc_mem),
                Some(&POINT::default()),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );

            GdipDisposeImage(image_ptr);
            GdipDeleteBrush(bg_brush_ptr);
            GdipDeletePen(bg_pen_ptr);
            GdipDeleteGraphics(graphics_ptr);

            let _ = DeleteObject(bitmap_icons.into());
            let _ = DeleteObject(bitmap_mem.into());
            let _ = DeleteDC(hdc_mem);
        }

        if self.show {
            return;
        }
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = SetFocus(Some(self.hwnd));
        }
        self.show = true;
    }

    pub fn unpaint(&mut self, _state: SwitchAppsState) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        self.show = false;
    }

    pub fn find_clicked_app_index(&self, state: &SwitchAppsState) -> Option<usize> {
        let cursor_pos = unsafe {
            let mut pos = POINT::default();
            let _ = GetCursorPos(&mut pos);
            pos
        };

        let dpi_scale = get_dpi_scale(self.hwnd);
        let icon_size_max = (ICON_SIZE_BASE as f64 * dpi_scale) as i32;
        let border_size = (WINDOW_BORDER_SIZE_BASE as f64 * dpi_scale) as i32;
        let icon_border = (ICON_BORDER_SIZE_BASE as f64 * dpi_scale) as i32;
        let icon_gap = (ICON_GAP_BASE as f64 * dpi_scale) as i32;
        let (label_height, _) = label_metrics(dpi_scale);

        let Coordinate {
            x,
            y,
            item_size,
            icon_cell,
            ..
        } = Coordinate::new(
            state.apps.len() as i32,
            icon_size_max,
            border_size,
            icon_border,
            icon_gap,
            label_height,
        );

        let xpos = cursor_pos.x - x;
        let ypos = cursor_pos.y - y;

        let cy = border_size;
        let item_height = icon_cell + label_height;
        for (i, _) in state.apps.iter().enumerate() {
            let cx = border_size + item_size * (i as i32);
            if xpos >= cx && xpos < cx + item_size && ypos >= cy && ypos < cy + item_height {
                return Some(i);
            }
        }
        None
    }
}

impl Drop for GdiAAPainter {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(Some(self.hwnd), self.hdc_screen);
            GdiplusShutdown(self.token);
        }
    }
}

const fn theme_color(light_theme: bool) -> (u32, u32) {
    match light_theme {
        true => (FG_LIGHT_COLOR, BG_LIGHT_COLOR),
        false => (FG_DARK_COLOR, BG_DARK_COLOR),
    }
}

unsafe fn draw_round_rect(
    graphic_ptr: *mut GpGraphics,
    brush_ptr: *mut GpBrush,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    corner_radius: f32,
) {
    unsafe {
        let mut path = GpPath::default();
        let mut path_ptr: *mut GpPath = &mut path;
        GdipCreatePath(FillModeAlternate, &mut path_ptr as _);
        GdipAddPathArc(
            path_ptr,
            left,
            top,
            corner_radius,
            corner_radius,
            180.0,
            90.0,
        );
        GdipAddPathArc(
            path_ptr,
            right - corner_radius,
            top,
            corner_radius,
            corner_radius,
            270.0,
            90.0,
        );
        GdipAddPathArc(
            path_ptr,
            right - corner_radius,
            bottom - corner_radius,
            corner_radius,
            corner_radius,
            0.0,
            90.0,
        );
        GdipAddPathArc(
            path_ptr,
            left,
            bottom - corner_radius,
            corner_radius,
            corner_radius,
            90.0,
            90.0,
        );
        GdipClosePathFigure(path_ptr);
        GdipFillPath(graphic_ptr, brush_ptr, path_ptr);
        GdipDeletePath(path_ptr);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_icons(
    state: &SwitchAppsState,
    hdc_screen: HDC,
    icon_size: i32,
    icon_border: i32,
    width: i32,
    height: i32,
    corner_radius: i32,
    fg_color: u32,
    bg_color: u32,
    item_size: i32,
    label_font_size: i32,
    text_color: u32,
) -> HBITMAP {
    let scaled_width = width * SCALE_FACTOR;
    let scaled_height = height * SCALE_FACTOR;
    let scaled_corner_radius = corner_radius * SCALE_FACTOR;
    let scaled_border_size = icon_border * SCALE_FACTOR;
    let scaled_icon_inner_size = icon_size * SCALE_FACTOR;
    let scaled_icon_cell = scaled_icon_inner_size + scaled_border_size * 2;
    let scaled_pitch = item_size * SCALE_FACTOR;
    // center the icon cell within its (wider) column pitch so the gap is even
    let cell_offset = (scaled_pitch - scaled_icon_cell) / 2;

    unsafe {
        let hdc_tmp = CreateCompatibleDC(Some(hdc_screen));
        let bitmap_tmp = CreateCompatibleBitmap(hdc_screen, width, height);
        SelectObject(hdc_tmp, bitmap_tmp.into());

        let hdc_scaled = CreateCompatibleDC(Some(hdc_screen));
        let bitmap_scaled = CreateCompatibleBitmap(hdc_screen, scaled_width, scaled_height);
        SelectObject(hdc_scaled, bitmap_scaled.into());

        let fg_brush = CreateSolidBrush(COLORREF(fg_color));
        let bg_brush = CreateSolidBrush(COLORREF(bg_color));

        let rect = RECT {
            left: 0,
            top: 0,
            right: scaled_width,
            bottom: scaled_height,
        };

        FillRect(hdc_scaled, &rect, bg_brush);

        for (i, (icon, _, _)) in state.apps.iter().enumerate() {
            let cell_left = scaled_pitch * (i as i32) + cell_offset;

            // draw the box for selected icon
            if i == state.index {
                let rgn = CreateRoundRectRgn(
                    cell_left,
                    0,
                    cell_left + scaled_icon_cell,
                    scaled_icon_cell,
                    scaled_corner_radius,
                    scaled_corner_radius,
                );
                let _ = FillRgn(hdc_scaled, rgn, fg_brush);
                let _ = DeleteObject(rgn.into());
            }

            let cx = cell_left + scaled_border_size;
            let _ = DrawIconEx(
                hdc_scaled,
                cx,
                scaled_border_size,
                *icon,
                scaled_icon_inner_size,
                scaled_icon_inner_size,
                0,
                None,
                DI_NORMAL,
            );
        }

        // draw the focused app's name under its own icon, within that icon's column
        let label_top = scaled_icon_cell;
        if scaled_height > label_top {
            if let Some((_, _, name)) = state.apps.get(state.index) {
                let mut text: Vec<u16> = name.encode_utf16().collect();
                if !text.is_empty() {
                    let face = w!("Segoe UI");
                    let hfont = CreateFontW(
                        -(label_font_size * SCALE_FACTOR),
                        0,
                        0,
                        0,
                        400,
                        0,
                        0,
                        0,
                        DEFAULT_CHARSET,
                        OUT_DEFAULT_PRECIS,
                        CLIP_DEFAULT_PRECIS,
                        ANTIALIASED_QUALITY,
                        0,
                        face,
                    );
                    let old_font = SelectObject(hdc_scaled, hfont.into());
                    SetBkMode(hdc_scaled, TRANSPARENT);
                    SetTextColor(hdc_scaled, COLORREF(text_color));
                    let cell_left = scaled_pitch * (state.index as i32) + cell_offset;
                    let mut rect = RECT {
                        left: cell_left,
                        top: label_top,
                        right: cell_left + scaled_icon_cell,
                        bottom: scaled_height,
                    };
                    DrawTextW(
                        hdc_scaled,
                        text.as_mut_slice(),
                        &mut rect,
                        DT_CENTER | DT_WORDBREAK | DT_NOPREFIX | DT_TOP,
                    );
                    SelectObject(hdc_scaled, old_font);
                    let _ = DeleteObject(hfont.into());
                }
            }
        }

        SetStretchBltMode(hdc_tmp, HALFTONE);
        let _ = StretchBlt(
            hdc_tmp,
            0,
            0,
            width,
            height,
            Some(hdc_scaled),
            0,
            0,
            scaled_width,
            scaled_height,
            SRCCOPY,
        );

        let _ = DeleteObject(fg_brush.into());
        let _ = DeleteObject(bg_brush.into());
        let _ = DeleteObject(bitmap_scaled.into());
        let _ = DeleteDC(hdc_scaled);
        let _ = DeleteDC(hdc_tmp);

        bitmap_tmp
    }
}

fn get_dpi_scale(hwnd: HWND) -> f64 {
    unsafe {
        let dpi = GetDpiForWindow(hwnd);
        if dpi == 0 {
            1.0
        } else {
            dpi as f64 / 96.0
        }
    }
}

struct Coordinate {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    icon_size: i32,
    /// Column pitch: icon cell plus the inter-icon gap.
    item_size: i32,
    /// Square that holds the icon and its selection highlight (no gap).
    icon_cell: i32,
}

impl Coordinate {
    fn new(
        num_apps: i32,
        icon_size_max: i32,
        border_size: i32,
        icon_border: i32,
        icon_gap: i32,
        label_height: i32,
    ) -> Self {
        let monitor_rect = get_moinitor_rect();
        let monitor_width = monitor_rect.right - monitor_rect.left;
        let monitor_height = monitor_rect.bottom - monitor_rect.top;

        let icon_size = ((monitor_width - 2 * border_size) / num_apps
            - icon_border * 2
            - icon_gap)
            .min(icon_size_max);

        let icon_cell = icon_size + icon_border * 2;
        let item_size = icon_cell + icon_gap;
        let width = item_size * num_apps + border_size * 2;
        let height = icon_cell + label_height + border_size * 2;
        let x = monitor_rect.left + (monitor_width - width) / 2;
        let y = monitor_rect.top + (monitor_height - height) / 2;

        Self {
            x,
            y,
            width,
            height,
            icon_size,
            item_size,
            icon_cell,
        }
    }
}

/// Returns `(label_height, label_font_size)` in device pixels for the given DPI scale.
fn label_metrics(dpi_scale: f64) -> (i32, i32) {
    let font_size = (LABEL_FONT_SIZE_BASE as f64 * dpi_scale) as i32;
    let line_height = (LABEL_LINE_HEIGHT_BASE as f64 * dpi_scale) as i32;
    let top_gap = (LABEL_TOP_GAP_BASE as f64 * dpi_scale) as i32;
    let label_height = top_gap + line_height * LABEL_LINES;
    (label_height, font_size)
}

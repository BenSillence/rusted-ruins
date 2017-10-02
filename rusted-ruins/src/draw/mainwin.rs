
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use array2d::*;
use common::basic::{TILE_SIZE, TILE_SIZE_I};
use common::objholder::Holder;
use common::obj::*;
use common::gobj;
use game::{Game, Animation, InfoGetter, CharaId};
use sdlvalues::SdlValues;

pub struct MainWinDrawer {
    rect: Rect,
    w: u32, h: u32,
    center_tile: Vec2d,
}

impl MainWinDrawer {
    pub fn new(rect: Rect) -> MainWinDrawer {
        MainWinDrawer {
            rect: rect,
            w: rect.width(), h: rect.height(),
            center_tile: Vec2d::new(0, 0),
        }
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, game: &Game, sv: &SdlValues,
                anim: Option<(&Animation, u32)>) {
        let mut player_move_dir = None;

        let player_move_adjust = if let Some(anim) = anim {
            match anim.0 {
                &Animation::PlayerMove{ n_frame, dir } => {
                    let v = dir.as_vec() * (TILE_SIZE_I * (n_frame - anim.1) as i32 / n_frame as i32);
                    player_move_dir = Some(dir);
                    (v.0, v.1)
                },
                _ => (0, 0)
            }
        }else{
            (0, 0)
        };
        
        self.draw_except_anim(canvas, game, sv, player_move_adjust, player_move_dir);

        if let Some(anim) = anim {
            self.draw_anim(canvas, game, sv, anim.0, anim.1);
        }
    }

    fn draw_except_anim(
        &mut self, canvas: &mut WindowCanvas, game: &Game, sv: &SdlValues,
        player_move_adjust: (i32, i32), player_move_dir: Option<Direction>) {

        canvas.set_viewport(self.rect);
        canvas.set_draw_color(Color::RGB(120, 120, 120));
        let current_map = &game.current_map;

        self.center_tile = game.player_pos();
        let (dx, dy) = self.calc_dxdy();
        let (dx, dy) = (dx + player_move_adjust.0, dy + player_move_adjust.1);
        let is_player_moving = player_move_adjust != (0, 0);

        let player_drawing_row = game.player_pos().1 + if let Some(dir) = player_move_dir {
            if dir.vdir == VDirection::Up { 1 }else{ 0 }
        }else{ 0 };
        
        let tile_range = self.tile_range();

        // Draw ground for the first row
        let top_row = tile_range.iter1().next().unwrap();
        for nx in tile_range.iter0() {
                let p = Vec2d::new(nx, top_row);
                self.draw_tile_ground(canvas, game, sv, p, dx, dy);
        }

        for ny in tile_range.iter1() {
            // Draw ground for the next row
            for nx in tile_range.iter0() {
                let p = Vec2d::new(nx, ny + 1);
                self.draw_tile_ground(canvas, game, sv, p, dx, dy);
            }

            // Draw player when moving
            if is_player_moving && ny == player_drawing_row {
                let chara = game.chara_holder.get(CharaId::Player);
                let ct = gobj::get_obj(chara.template_idx);
                let src = Rect::from(ct.img_rect());
                let dest = centering_at_tile(src, game.player_pos(), dx - player_move_adjust.0, dy - player_move_adjust.1);
                canvas.copy(sv.tex().get(chara.template_idx), src, dest).unwrap();
            }

            for nx in tile_range.iter0() {
                let p = Vec2d::new(nx, ny);
                
                // Draw wall
                self.draw_tile_wall(canvas, game, sv, p, dx, dy);

                if !current_map.is_inside(p) { continue; }

                // Draw character on the tile
                if let Some(chara_id) = current_map.get_chara(p) {
                    let chara = game.chara_holder.get(chara_id);
                    let ct = gobj::get_obj(chara.template_idx);
                    let src = Rect::from(ct.img_rect());
                    
                    if chara_id == CharaId::Player && is_player_moving {
                        continue;
                    };
                    
                    let dest = if chara_id == CharaId::Player {
                        centering_at_tile(src, p, dx - player_move_adjust.0, dy - player_move_adjust.1)
                    }else{
                        centering_at_tile(src, p, dx, dy)
                    };
                    canvas.copy(sv.tex().get(chara.template_idx), src, dest).unwrap();
                }
            }
        }
    }
    
    fn draw_tile_ground(
        &mut self, canvas: &mut WindowCanvas, game: &Game, sv: &SdlValues,
        p: Vec2d, dx: i32, dy: i32) {
        let current_map = &game.current_map;
        if !current_map.is_inside(p) { return; }
        let dest = Rect::new(
            p.0 * TILE_SIZE_I + dx, p.1 * TILE_SIZE_I + dy,
            TILE_SIZE, TILE_SIZE);
        let texture = sv.tex().get(current_map.tile[p].tile);
        check_draw!(canvas.copy(&texture, None, dest));
    }

    fn draw_tile_wall(
        &mut self, canvas: &mut WindowCanvas, game: &Game, sv: &SdlValues,
        p: Vec2d, dx: i32, dy: i32) {
        
        let current_map = &game.current_map;
        let wall_idx = if current_map.is_inside(p) {
            match current_map.tile[p].wall {
                Some(wall_idx) => wall_idx, None => { return; }
            }
        }else{
            ::common::objholder::WallIdx(0)
        };
        let wall = gobj::get_obj(wall_idx);
        let src = Rect::from(wall.img_rect());
        let dest = bottom_at_tile(src, p, dx, dy);
        let texture = sv.tex().get(wall_idx);
        check_draw!(canvas.copy(&texture, src, dest));
    }

    fn draw_anim(&mut self, canvas: &mut WindowCanvas, game: &Game, sv: &SdlValues,
                 anim: &Animation, i_frame: u32) {
        
        let (dx, dy) = self.calc_dxdy();

        match anim {
            &Animation::Img{ idx, range, .. } => {
                for p in range {
                    let src = Rect::from(gobj::get_obj(idx).img_rect_nth(i_frame));
                    let dest = centering_at_tile(src, p, dx, dy);
                    check_draw!(canvas.copy(sv.tex().get(idx), src, dest));
                }
            }
            _ => (),
        }
    }

    /// Calculates adjustment value for centering
    fn calc_dxdy(&self) -> (i32, i32) {
        (
            (self.w / 2) as i32 - self.center_tile.0 * TILE_SIZE_I - TILE_SIZE_I / 2,
            (self.h / 2) as i32 - self.center_tile.1 * TILE_SIZE_I - TILE_SIZE_I / 2,
        )
    }

    /// Gets needed range of tiles to draw over the window
    fn tile_range(&self) -> RectIter {
        let w_win = self.w as i32; let h_win = self.h as i32;
        let ct = self.center_tile;
        let n0 = (w_win - TILE_SIZE_I) / (2 * TILE_SIZE_I) + 1;
        let n1 = (h_win - TILE_SIZE_I) / (2 * TILE_SIZE_I) + 1;
        let top_left     = (ct.0 - n0 - 1, ct.1 - n1 - 1);
        let bottom_right = (ct.0 + n0 + 1, ct.1 + n1 + 2);
        RectIter::new(top_left, bottom_right)
    }
}

#[inline]
fn centering_at_tile(src: Rect, tile: Vec2d, dx: i32, dy: i32) -> Rect {
    Rect::new(
        (TILE_SIZE_I * tile.0 + (TILE_SIZE_I - src.w) / 2) + dx,
        (TILE_SIZE_I * tile.1 + (TILE_SIZE_I - src.h) / 2) + dy,
        src.w as u32, src.h as u32
    )
}

#[inline]
fn bottom_at_tile(src: Rect, tile: Vec2d, dx: i32, dy: i32) -> Rect {
    Rect::new(
        (TILE_SIZE_I * tile.0 + (TILE_SIZE_I - src.w) / 2) + dx,
        tile.1 * TILE_SIZE_I + dy + (TILE_SIZE_I - src.h),
        src.w as u32, src.h as u32
    )
}


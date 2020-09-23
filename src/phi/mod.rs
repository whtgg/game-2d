#[macro_use]
mod events;

events_macro!{
    keyboard: {
        key_escape: Escape,
        key_up:Up,
        key_down:Down,
        key_left:Left,
        key_right:Right,
        key_space:Space
    },
    else: {
        quit: Quit{ .. }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rectangle {
    pub x:f64,
    pub y:f64,
    pub w:f64,
    pub h:f64
}
use sdl2::rect::Rect as SdlRect;
impl Rectangle {
    pub fn new(x:f64,y:f64,w:f64,h:f64) -> Self {
        Self {x,y,w,h}
    }

    pub fn to_sdl(&self) -> Option<SdlRect> {
        assert!(self.w >= 0.0 && self.h >= 0.0);
        
        Some(SdlRect::new(
            self.x as i32,
            self.y as i32,
            self.w as u32,
            self.h as u32
        ))
    }

    pub fn contains(&self,rect:Rectangle) -> bool {
        let x_min = rect.x;
        let x_max = x_min + rect.w;
        let y_min = rect.y;
        let y_max = y_min + rect.h;

        x_min >= self.x && x_min <= self.x + self.w && x_max >= self.x &&
            x_max <= self.x + self.w && y_min >= self.y
            && y_min <= self.y + self.h &&
            y_max >= self.y && y_max <= self.y + self.h
    }


    pub fn move_inside(self,parent:Rectangle) -> Option<Rectangle> {
        if self.w > parent.w || self.h > parent.h {
            return None;
        }
        Some(Rectangle{
            w: self.w,
            h: self.h,
            x: if self.x < parent.x {
                parent.x
            } else if self.x + self.w >= parent.x + parent.w {
                parent.x + parent.w - self.w
            } else {
                self.x
            },
            y: if self.y < parent.y {
                parent.y
            } else if self.y + self.h >= parent.y + parent.h {
                parent.y + parent.h - self.h
            } else {
                self.y
            },
        })
    }
}

use sdl2::image::{LoadTexture};
use sdl2::render::{Texture,TextureCreator,TextureQuery};
use sdl2::video::WindowContext;
use std::rc::Rc;
use std::cell::RefCell;



struct Ship<'a>{
    rect:Rectangle,
    sprites:Vec<Sprite<'a>>,
    current:ShipFrame,
    // tex:Texture<'a>,
}

#[derive(Clone)]
pub struct Sprite<'a> {
    tex:Rc<RefCell<Texture<'a>>>,
    src:Rectangle,
}

#[derive(Clone,Copy,Debug)]
enum ShipFrame {
    UpNorm = 0,
    UpFast = 1,
    UpSlow = 2,
    MidNorm = 3,
    MidFast = 4,
    MidSlow = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8,
}


// struct Rect {
//     rect:Rectangle
// }

pub struct ShipView<'a> {
    player:Ship<'a>,
    bg_back:Background<'a>,
    bg_middle:Background<'a>,
    bg_front:Background<'a>,
}

#[derive(Clone)]
pub struct Background<'a> {
    pos: f64,
    vel: f64,
    sprite: Sprite<'a>,
}


impl <'a> Background<'a> {
    fn render(&mut self,canvas:&mut WindowCanvas,elapsed:f64) {
        let (w,h) = self.sprite.size();
        self.pos += self.vel * elapsed;
        
        if self.pos > w {
            self.pos -= w
        }

        let (win_w,win_h) = canvas.output_size().unwrap();
        let scale = win_h as f64 / h;
        
        let mut physical_left = -self.pos * scale;
        
        while physical_left < win_w as f64 {
            canvas.copy_sprite(&self.sprite, Rectangle{
                x:physical_left,
                y:0.0,
                w:w*scale,
                h:win_h as f64
            });
            physical_left += w * scale;
        }
        //physical_left += w * scale;
        
    } 
}

use std::path::Path;
impl <'a> Sprite <'a> {
    pub fn new(t:Texture<'a>) -> Self{
        //let tex:Texture = t.load_texture("./assets/spaceship.png").unwrap();
        let TextureQuery {width,height, .. } = t.query();
        Self {
            src:Rectangle {
                w:width  as f64,
                h:height as f64,
                x:0.0,
                y:0.0,
            },
            tex:Rc::new(RefCell::new(t))
        }
    }

    pub fn load(t:&'a TextureCreator<WindowContext>,path:&str) ->Option<Sprite<'a>> {
        t.load_texture(Path::new(path)).ok().map(Sprite::new)
    }

    pub fn region(&self, rect: Rectangle) -> Option<Sprite<'a>> {
        let src: Rectangle = Rectangle {
            x: rect.x + self.src.x,
            y: rect.y + self.src.y,
            ..rect
        };

        if self.src.contains(src) {
            Some(Sprite {
                tex: self.tex.clone(),
                src,
            })
        } else {
            None
        }
    }

    pub fn render(&self,canvas:&mut WindowCanvas,dest:Rectangle) {
        canvas.copy(&mut self.tex.borrow_mut(),
            self.src.to_sdl(), dest.to_sdl())
            .expect("failed to copy texture");
    }

    pub fn size(&self) -> (f64,f64) {
        (self.src.w,self.src.h)
    }
}

const PLAYER_SPEED:f64 = 0.32;
const SHIP_W:f64 = 43.0;
const SHIP_H:f64 = 39.0;

impl <'a> ShipView<'a> {
    pub fn new(ct:&'a TextureCreator<WindowContext>) -> Self {
        let sprite_sheet = Sprite::load(ct, "./assets/spaceship.png").unwrap();
        
        let mut sprites:Vec<Sprite> = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(
                    sprite_sheet
                        .region(Rectangle {
                            w: SHIP_W,
                            h: SHIP_H,
                            x: SHIP_W * x as f64,
                            y: SHIP_H * y as f64,
                        })
                        .unwrap(),
                )
            }
        }

        Self {
            player:Ship{
                rect:Rectangle {
                    x:64.0,
                    y:64.0,
                    w:32.0,
                    h:32.0,
                },
                sprites,
                current:ShipFrame::MidNorm,
            },
            bg_front:Background{
                pos:0.0,
                vel:80.0,
                sprite:Sprite::load(ct,"./assets/starFG.png").unwrap(),
            },
            bg_middle:Background {
                pos:0.0,
                vel:40.0,
                sprite:Sprite::load(ct, "./assets/starMG.png").unwrap(),
            },
            bg_back:Background {
                pos:0.0,
                vel:40.0,
                sprite:Sprite::load(ct, "./assets/starBG.png").unwrap(),
            }
        }
    }
}

use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window as Win;



// Trait to render a sprite within an area
pub trait CopySprite {
    fn copy_sprite(&mut self, sprite: &Sprite, dest: Rectangle);
}

// impl <'window> CopySprite for WindowCanvas{
//     fn copy_sprite(&mut self, renderable: &Sprite, dest: Rectangle){
//         renderable.render(self, dest);
//     }
// }

impl CopySprite for Canvas<Win> {
    fn copy_sprite(&mut self, renderable: &Sprite, dest: Rectangle){
        renderable.render(self, dest);
    }
}




impl <'a> View for ShipView<'a>{
    fn render(&mut self,context:&mut Phi,elapsed:f64) -> ViewAction {
        //w/h
        let (w,h) = context.canvas.output_size().unwrap();
        let canvas = &mut context.canvas;
        let events = &mut context.events;

        if events.now.quit || events.now.key_escape == Some(true) {
            return  ViewAction::Quit;
        }

        //斜对角
        let digonal:bool = (events.key_up ^ events.key_up) && (events.key_left ^ events.key_right);

        //偏移距离
        let moved = if digonal { 1.0/2.0f64.sqrt()} else {1.0} * PLAYER_SPEED;

        //x轴偏移量
        let dx = match (events.key_left,events.key_right) { 
            (true,true) | (false,false) => 0.0,
            (true,false) => -moved,
            (false,true) => moved
        };
        
        //y轴偏移量
        let dy = match (events.key_up,events.key_down) {
            (true,true) | (false,false) => 0.0,
            (true,false) => -moved,
            (false,true) => moved
        };

        self.player.rect.x += dx;
        self.player.rect.y += dy;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(200,200,50));


        let moveable_region:Rectangle = Rectangle::new(0.0,0.0,w as f64 * 0.7,h as f64);
        
        self.player.rect = self.player.rect.move_inside(moveable_region).unwrap();


        self.bg_back.render(canvas, elapsed);
        self.bg_middle.render(canvas, elapsed);
        

        self.player.current =
            if dx == 0.0 && dy < 0.0 { ShipFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0 { ShipFrame::UpFast }
            else if dx < 0.0 && dy < 0.0 { ShipFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { ShipFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0 { ShipFrame::MidFast }
            else if dx < 0.0 && dy == 0.0 { ShipFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0 { ShipFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0 { ShipFrame::DownFast }
            else if dx < 0.0 && dy > 0.0 { ShipFrame::DownSlow }
            else { unreachable!() };

        // self.player.sprites[self.player.current as usize]
        //     .render(canvas, self.player.rect);
        
        //canvas.
        canvas.copy_sprite(&self.player.sprites[self.player.current as usize], self.player.rect);
        self.bg_front.render(canvas, elapsed);
        ViewAction::None
    }
}





pub struct Phi {
    pub events: Events,
    pub canvas: WindowCanvas,
}

impl Phi {
    pub fn new(events:Events,canvas:WindowCanvas) -> Self {
        Self {
            events,
            canvas,
        }
    }

    // pub fn output_size(&self) -> (f64,f64) {
    //     self.canvas.output_size()
    // }
}





pub enum ViewAction {
    Quit,
    None,
    //ChangeView(Box<dyn View>),
}


pub trait View {
    fn render(&mut self, context: &mut Phi,elapsed: f64) -> ViewAction;
}


// pub struct ViewA;
// pub struct ViewB;



// impl View for ViewA {
//     fn render(&mut self,context:&mut Phi) -> ViewAction {
//         let canvas = &mut context.canvas;
//         let events  = &mut context.events;
        
//         if events.now.quit || events.now.key_escape == Some(true) {
//             return  ViewAction::Quit;
//         }

//         if let Some(true) = events.now.key_space {
//             return ViewAction::ChangeView(Box::new(ViewB));
//         }
        
//         canvas.set_draw_color(Color::RGB(0, 0, 0));
//         canvas.clear();
//         ViewAction::None
//     }
// }

// impl View for ViewB {
//     fn render(&mut self,context:&mut Phi) -> ViewAction {
//         let canvas = &mut context.canvas;
//         let events = &mut context.events;
        
//         if events.now.quit || events.now.key_escape == Some(true) {
//             return ViewAction::Quit;
//         }

//         if let Some(true) = events.now.key_space {
//             return ViewAction::ChangeView(Box::new(ViewA));
//         }

//         canvas.set_draw_color(Color::RGB(111, 111, 111));
//         canvas.clear();
//         ViewAction::None
//     }
// }

pub fn spawn(title:&str) {
    let sdl2_context = sdl2::init().unwrap();
    let video = sdl2_context.video().unwrap();
    let window = video
            .window(title,800,600)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

    let canvas = window.into_canvas().software().build().unwrap();

    let events = Events::new(sdl2_context.event_pump().unwrap());
    
    let mut context = Phi::new(events,canvas);
    //不能直接引用
    let mut creat = context.canvas.texture_creator();
    let mut current_view:Box<dyn View> = Box::new(ShipView::new(&mut creat));

    let mut timer = sdl2_context.timer().unwrap();
    let interval = 1000/60;
    let mut before  = timer.ticks();

    'running: loop {
        let now = timer.ticks();
        let dt = now -before;
        let elapased = dt as f64 / 1000.0;
        
        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        context.events.pump();
        match current_view.render(&mut context,elapased) {
            ViewAction::Quit => break 'running,
            ViewAction::None => context.canvas.present(),
            //ViewAction::ChangeView(view)=>current_view = view
        }
    }
}

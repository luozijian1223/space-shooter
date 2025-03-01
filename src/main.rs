use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Color, DrawParam, Mesh, Rect},
    input::keyboard::{KeyCode, KeyInput},
    timer, Context, GameError, GameResult,
};
use rand::{self, Rng};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PLAYER_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 400.0;
const ENEMY_SPEED: f32 = 100.0;
const ENEMY_SPAWN_INTERVAL: f32 = 1.0;

struct GameObject {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    alive: bool,
}

impl GameObject {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            size: Vec2::new(width, height),
            alive: true,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(
            self.position.x - self.size.x / 2.0,
            self.position.y - self.size.y / 2.0,
            self.size.x,
            self.size.y,
        )
    }

    fn collides_with(&self, other: &GameObject) -> bool {
        self.bounds().overlaps(&other.bounds())
    }
}

struct MainState {
    player: GameObject,
    bullets: Vec<GameObject>,
    enemies: Vec<GameObject>,
    score: u32,
    game_over: bool,
    spawn_timer: f32,
}

impl MainState {
    fn new() -> Self {
        let player = GameObject::new(
            WINDOW_WIDTH / 2.0,
            WINDOW_HEIGHT - 50.0,
            30.0,
            30.0,
        );

        Self {
            player,
            bullets: Vec::new(),
            enemies: Vec::new(),
            score: 0,
            game_over: false,
            spawn_timer: 0.0,
        }
    }

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(20.0..WINDOW_WIDTH - 20.0);
        
        let enemy = GameObject::new(x, -20.0, 30.0, 30.0);
        self.enemies.push(enemy);
    }

    fn fire_bullet(&mut self) {
        let bullet = GameObject {
            position: self.player.position - Vec2::new(0.0, 20.0),
            velocity: Vec2::new(0.0, -BULLET_SPEED),
            size: Vec2::new(5.0, 10.0),
            alive: true,
        };
        
        self.bullets.push(bullet);
    }

    fn reset(&mut self) {
        *self = MainState::new();
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx).as_secs_f32();

        if self.game_over {
            if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                self.reset();
            }
            return Ok(());
        }

        // 更新玩家位置
        self.player.position += self.player.velocity * dt;
        
        // 保持玩家在屏幕内
        self.player.position.x = self.player.position.x.clamp(
            self.player.size.x / 2.0, 
            WINDOW_WIDTH - self.player.size.x / 2.0
        );

        // 更新子弹位置
        for bullet in &mut self.bullets {
            bullet.position += bullet.velocity * dt;
            
            // 删除离开屏幕的子弹
            if bullet.position.y < -10.0 {
                bullet.alive = false;
            }
        }
        self.bullets.retain(|bullet| bullet.alive);

        // 更新敌人位置
        for enemy in &mut self.enemies {
            enemy.position.y += ENEMY_SPEED * dt;
            
            // 游戏结束条件：敌人到达底部
            if enemy.position.y > WINDOW_HEIGHT + 15.0 {
                self.game_over = true;
            }

            // 检测玩家与敌人碰撞
            if self.player.collides_with(enemy) {
                self.game_over = true;
            }
        }

        // 检测子弹与敌人碰撞
        for bullet in &mut self.bullets {
            for enemy in &mut self.enemies {
                if bullet.collides_with(enemy) && enemy.alive {
                    bullet.alive = false;
                    enemy.alive = false;
                    self.score += 10;
                }
            }
        }
        self.enemies.retain(|enemy| enemy.alive);

        // 生成新敌人
        self.spawn_timer += dt;
        if self.spawn_timer >= ENEMY_SPAWN_INTERVAL {
            self.spawn_enemy();
            self.spawn_timer = 0.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // 绘制玩家
        let player_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.player.bounds(),
            Color::WHITE,
        )?;
        canvas.draw(&player_mesh, DrawParam::default());

        // 绘制子弹
        for bullet in &self.bullets {
            let bullet_mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                bullet.bounds(),
                Color::YELLOW,
            )?;
            canvas.draw(&bullet_mesh, DrawParam::default());
        }

        // 绘制敌人
        for enemy in &self.enemies {
            let enemy_mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                enemy.bounds(),
                Color::RED,
            )?;
            canvas.draw(&enemy_mesh, DrawParam::default());
        }

        // 绘制分数
        let score_text = graphics::Text::new(format!("分数: {}", self.score));
        canvas.draw(
            &score_text,
            DrawParam::default().dest(Vec2::new(10.0, 10.0)),
        );

        // 游戏结束提示
        if self.game_over {
            let game_over_text = graphics::Text::new("游戏结束! 按R键重新开始");
            canvas.draw(
                &game_over_text,
                DrawParam::default().dest(Vec2::new(
                    WINDOW_WIDTH / 2.0 - 120.0,
                    WINDOW_HEIGHT / 2.0,
                )),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult<()> {
        if self.game_over {
            return Ok(());
        }

        match input.keycode {
            Some(KeyCode::Left) => self.player.velocity.x = -PLAYER_SPEED,
            Some(KeyCode::Right) => self.player.velocity.x = PLAYER_SPEED,
            Some(KeyCode::Space) => self.fire_bullet(),
            _ => (),
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult<()> {
        match input.keycode {
            Some(KeyCode::Left) | Some(KeyCode::Right) => {
                self.player.velocity.x = 0.0;
            }
            _ => (),
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ggez::ContextBuilder::new("space_shooter", "luozijian1223")
        .window_setup(WindowSetup::default().title("太空射击游戏"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;

    let state = MainState::new();
    event::run(ctx, event_loop, state)
}
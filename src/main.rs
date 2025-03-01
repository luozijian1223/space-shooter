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

struct Player {
    game_object: GameObject,
    lives: u32,
    invincible_timer: f32,  // 受伤后的短暂无敌时间
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            game_object: GameObject::new(x, y, 30.0, 30.0),
            lives: 3,  // 初始3条命
            invincible_timer: 0.0,
        }
    }
    
    // 当玩家受到伤害时调用
    fn take_damage(&mut self) -> bool {
        if self.invincible_timer <= 0.0 {
            self.lives -= 1;
            self.invincible_timer = 2.0;  // 2秒无敌时间
            return true;
        }
        false
    }
    
    // 更新玩家状态，包括无敌时间
    fn update(&mut self, dt: f32) {
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }
    }
    
    // 检查玩家是否处于无敌状态
    fn is_invincible(&self) -> bool {
        self.invincible_timer > 0.0
    }
}

struct MainState {
    player: Player,
    bullets: Vec<GameObject>,
    enemies: Vec<GameObject>,
    powerups: Vec<GameObject>,  // 新增道具列表
    score: u32,
    game_over: bool,
    spawn_timer: f32,
    powerup_timer: f32,  // 道具生成计时器
}

impl MainState {
    fn new() -> Self {
        let player = Player::new(
            WINDOW_WIDTH / 2.0,
            WINDOW_HEIGHT - 50.0,
        );

        Self {
            player,
            bullets: Vec::new(),
            enemies: Vec::new(),
            powerups: Vec::new(),  // 初始化为空列表
            score: 0,
            game_over: false,
            spawn_timer: 0.0,
            powerup_timer: 0.0,
        }
    }

    // 添加生成道具的方法
    fn spawn_powerup(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(20.0..WINDOW_WIDTH - 20.0);
        
        let powerup = GameObject::new(x, -20.0, 20.0, 20.0);
        self.powerups.push(powerup);
    }
    

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(20.0..WINDOW_WIDTH - 20.0);
        
        let enemy = GameObject::new(x, -20.0, 30.0, 30.0);
        self.enemies.push(enemy);
    }

    fn fire_bullet(&mut self) {
        let bullet = GameObject {
            position: self.player.game_object.position - Vec2::new(0.0, 20.0),
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

        // 检查游戏是否结束（生命值为0）
        if self.game_over {
            if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                self.reset();
            }
            return Ok(());
        }

        // 更新玩家状态，包括无敌时间
        self.player.update(dt);

        // 更新玩家位置
        self.player.game_object.position += self.player.game_object.velocity * dt;
        
        // 保持玩家在屏幕内
        self.player.game_object.position.x = self.player.game_object.position.x.clamp(
            self.player.game_object.size.x / 2.0, 
            WINDOW_WIDTH - self.player.game_object.size.x / 2.0
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
            
            // 敌人到达底部，玩家损失一条命
            if enemy.position.y > WINDOW_HEIGHT + 15.0 {
                enemy.alive = false;
                if self.player.take_damage() && self.player.lives == 0 {
                    self.game_over = true;
                }
            }

            // 检测玩家与敌人碰撞
            if !self.player.is_invincible() && 
               self.player.game_object.collides_with(enemy) {
                enemy.alive = false;
                if self.player.take_damage() && self.player.lives == 0 {
                    self.game_over = true;
                }
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

        // 绘制玩家，无敌时闪烁效果
        if !self.player.is_invincible() || 
           (self.player.is_invincible() && (self.player.invincible_timer * 10.0) as i32 % 2 == 0) {
            
            let player_color = if self.player.is_invincible() {
                Color::new(1.0, 1.0, 0.5, 0.8)  // 受伤后呈现黄色半透明
            } else {
                Color::WHITE
            };
            
            let player_mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    self.player.game_object.position.x - self.player.game_object.size.x / 2.0,
                    self.player.game_object.position.y - self.player.game_object.size.y / 2.0,
                    self.player.game_object.size.x,
                    self.player.game_object.size.y,
                ),
                player_color,
            )?;
            canvas.draw(&player_mesh, DrawParam::default());
        }

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
        
        // 绘制生命值
        let lives_text = graphics::Text::new(format!("生命: {}", self.player.lives));
        canvas.draw(
            &lives_text,
            DrawParam::default().dest(Vec2::new(10.0, 40.0)),
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

    // 修改key_down_event和key_up_event以使用player.game_object
    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult<()> {
        if self.game_over {
            return Ok(());
        }

        match input.keycode {
            Some(KeyCode::Left) => self.player.game_object.velocity.x = -PLAYER_SPEED,
            Some(KeyCode::Right) => self.player.game_object.velocity.x = PLAYER_SPEED,
            Some(KeyCode::Space) => self.fire_bullet(),
            _ => (),
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult<()> {
        match input.keycode {
            Some(KeyCode::Left) | Some(KeyCode::Right) => {
                self.player.game_object.velocity.x = 0.0;
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
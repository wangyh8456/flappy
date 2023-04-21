use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH:i32=80;
const SCREEN_HEIGHT:i32=50;
//每隔70ms更新一次
const FRAME_DURATION:f32=70.0;

struct Player{
    x:i32,
    y:i32,
    velocity:f32
}
impl Player{
    fn new(x:i32,y:i32)->Self{
        Self{
            x,
            y,
            //下面这种写法根本不会取输入的y值，而是直接取0
            // y:0,
            velocity:0.0
        }
    }
    fn render(&mut self,ctx:&mut BTerm){
        //x:0仅表示绘制在屏幕的最左侧，不是世界坐标，因此障碍物坐标减去玩家世界坐标，就能得到相对于玩家的坐标，即相对于屏幕的坐标
        ctx.set(0,self.y,YELLOW,BLACK,to_cp437('@'));
    }
    fn gravity_and_move(&mut self){
        //velocity大于0表示的是龙往下掉，速度小于2时，向下的加速度为0.2，往下掉的越来越快，大于等于2之后速度不变
        if self.velocity<2.0{
            self.velocity+=0.2;
        }
        //等于速度乘以1加到纵坐标上
        self.y+=self.velocity as i32;
        //单位时间内，龙向右移动1个单位
        self.x+=1;
        //如果龙跑到屏幕上面，就让他回到屏幕顶部y=0处
        if self.y<0{
            self.y=0;
        }
    }

    fn flap(&mut self){
        //当龙往下掉时，按空格键，速度变为-2，往上飞
        //纵坐标减2，往上飞，横坐标加1，往右飞
        self.velocity=-2.0;
    }
}

struct State {
    mode: GameMode,
    player:Player,
    frame_time:f32,
    score:i32,
    obstacle:Obstacle,
}
impl State {
    fn new() -> Self {
        Self {
            mode: GameMode::Menu,
            player:Player::new(5,25),
            frame_time:0.0,
            score:0,
            //初始障碍物在最右边
            obstacle:Obstacle::new(SCREEN_WIDTH,0),
        }
    }
    fn play(&mut self,ctx:&mut BTerm) {
        ctx.cls_bg(NAVY);
        //当状态为playing时每次tick累积时间，当达到设置的FRAME_DURATION时，调用player的gravity_and_move方法进行更新
        self.frame_time+=ctx.frame_time_ms;

        if(self.frame_time>FRAME_DURATION){
            self.frame_time=0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space)=ctx.key{
            self.player.flap();
        }
        //其中x为0，表示每次渲染玩家都在最左侧，能连续玩游戏
        self.player.render(ctx);
        ctx.print(0,0,"Press [Space] to flap!");
        ctx.print(0,1,&format!("Score: {}",self.score));

        self.obstacle.render(ctx, self.player.x);
        if self.player.x>self.obstacle.x{
            self.score+=1;
            //越过障碍物后始终在屏幕最右侧生成新的障碍物
            self.obstacle=Obstacle::new(self.player.x+SCREEN_WIDTH,self.score);
        }
        
        if self.player.y>SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player){
            self.mode=GameMode::End;
        }
    }
    fn restart(&mut self) {
        self.mode=GameMode::Playing;
        self.player=Player::new(5,25);
        self.frame_time=0.0;
        self.score=0;
        self.obstacle=Obstacle::new(SCREEN_WIDTH,0);
    }
    fn main_menu(&mut self,ctx:&mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon!");
        ctx.print_centered(8, "[P] Play Game");
        ctx.print_centered(9, "[Q] Quit Game");

        if let Some(key)=ctx.key{
            match key{
                VirtualKeyCode::P=>self.restart(),
                VirtualKeyCode::Q=>ctx.quitting=true,
                _=>{}
            }
        }
    }

    fn dead(&mut self,ctx:&mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points.",self.score));
        ctx.print_centered(8, "[P] Play Game");
        ctx.print_centered(9, "[Q] Quit Game");

        if let Some(key)=ctx.key{
            match key{
                VirtualKeyCode::P=>self.restart(),
                VirtualKeyCode::Q=>ctx.quitting=true,
                _=>{}
            }
        }
    }
}

impl GameState for State{
    fn tick(&mut self, ctx:&mut BTerm) {
       match self.mode{
            GameMode::Menu=>self.main_menu(ctx),
            GameMode::Playing=>self.play(ctx),
            GameMode::End=>self.dead(ctx),
       }
    }
}

struct Obstacle{
    x:i32,
    gap_y:i32,
    size:i32,
}
impl Obstacle{
    fn new(x:i32,score:i32)->Self{
        //伪随机数生成器
        let mut random=RandomNumberGenerator::new();
        Self{
            x,
            //随机生成障碍物的y坐标，范围为10-40，不包含40
            gap_y:random.range(10,40),
            size:i32::max(2,20-score),
        }
    }

    fn render(&mut self,ctx:&mut BTerm,player_x:i32){
        //self_x和player_x为世界坐标，相减之后得到的screen_x为屏幕坐标
        let screen_x=self.x-player_x;
        let half_size=self.size/2;
        //y是整形，绘制时按照高度1，x为screen_x，颜色为红色，背景为黑色，字符为|,一直到绘制出整个高度
        for y in 0..self.gap_y-half_size{
            ctx.set(screen_x,y,RED,BLACK,to_cp437('|'));
        }
        for y in self.gap_y+half_size..SCREEN_HEIGHT{
            ctx.set(screen_x,y,RED,BLACK,to_cp437('|'));
        }
    }
    fn hit_obstacle(&self,player:&Player)->bool{
        let half_size=self.size/2;
        player.x==self.x&&
        (player.y<self.gap_y-half_size||player.y>self.gap_y+half_size)
    }
}

fn main() -> BError {
    let context=BTermBuilder::simple80x50()
        .with_title("Flappy Dragon!")
        .build()?;//问号表示若构建错误返回BError
    
    main_loop(context, State::new())
}

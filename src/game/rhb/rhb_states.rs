use crate::{
    engine::{Audio, Point, Sound},
    game::HEIGHT,
};

const FLOOR: i16 = 479;
const PLAYER_HEIGHT: i16 = HEIGHT - FLOOR;
const STARTING_POINT: i16 = -20;
const RUNNING_SPEED: i16 = 4;
const JUMP_SPEED: i16 = -25;
const GRAVITY: i16 = 1;
const TERMINAL_VELOCITY: i16 = 20;

const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
const SLIDING_FRAMES: u8 = 14;
const JUMPING_FRAMES: u8 = 35;
const FALLING_FRAMES: u8 = 29;
const IDLE_FRAME_NAME: &str = "Idle";
const RUN_FRAME_NAME: &str = "Run";
const FALLING_FRAME_NAME: &str = "Dead";
const SLIDING_FRAME_NAME: &str = "Slide";
const JUMPING_FRAME_NAME: &str = "Jump";

#[derive(Clone)]
pub struct RedHatBoyState<S> {
    context: RedHatBoyContext,
    _state: S,
}

impl<S> RedHatBoyState<S> {
    pub fn context(&self) -> &RedHatBoyContext {
        &self.context
    }

    pub fn update_context(&mut self, frames: u8) {
        self.context = self.context.clone().update(frames);
    }
}

#[derive(Clone)]
pub struct RedHatBoyContext {
    pub frame: u8,
    pub position: Point,
    pub velocity: Point,
    pub audio: Audio,
    pub jump_sound: Sound,
}

impl RedHatBoyContext {
    pub fn update(mut self, frame_count: u8) -> Self {
        if self.velocity.y < TERMINAL_VELOCITY {
            self.velocity.y += GRAVITY;
        }

        if self.frame < frame_count {
            self.frame += 1;
        } else {
            self.frame = 0;
        }

        self.position.y += self.velocity.y;

        if self.position.y > FLOOR {
            self.position.y = FLOOR;
        }

        self
    }

    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        self
    }

    fn run_right(mut self) -> Self {
        self.velocity.x += RUNNING_SPEED;
        self
    }

    fn set_vertical_velocity(mut self, y: i16) -> Self {
        self.velocity.y = y;
        self
    }

    fn stop(mut self) -> Self {
        self.velocity.x = 0;
        self.velocity.y = 0;
        self
    }

    fn set_on(mut self, position: i16) -> Self {
        let position = position - PLAYER_HEIGHT;
        self.position.y = position;
        self
    }

    fn play_jump_sound(self) -> Self {
        if let Err(err) = self.audio.play_sound(&self.jump_sound) {
            log!("Error playing jump sound {:#?}", err);
        }
        self
    }
}

#[derive(Copy, Clone)]
pub struct Idle;

impl RedHatBoyState<Idle> {
    pub fn frame_name(&self) -> &str {
        IDLE_FRAME_NAME
    }

    pub fn update(mut self) -> Self {
        self.update_context(IDLE_FRAMES);
        self
    }

    pub fn new(audio: Audio, jump_sound: Sound) -> Self {
        RedHatBoyState {
            context: RedHatBoyContext {
                frame: 0,
                position: Point {
                    x: STARTING_POINT,
                    y: FLOOR,
                },
                velocity: Point { x: 0, y: 0 },
                audio,
                jump_sound,
            },
            _state: Idle {},
        }
    }

    pub fn run(self) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.reset_frame().run_right(),
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
pub struct Running;

impl RedHatBoyState<Running> {
    pub fn frame_name(&self) -> &str {
        RUN_FRAME_NAME
    }

    pub fn update(mut self) -> Self {
        self.update_context(RUNNING_FRAMES);
        self
    }

    pub fn slide(self) -> RedHatBoyState<Sliding> {
        RedHatBoyState {
            context: self.context.reset_frame(),
            _state: Sliding {},
        }
    }

    pub fn knock_out(self) -> RedHatBoyState<Falling> {
        RedHatBoyState {
            context: self.context.reset_frame().stop(),
            _state: Falling {},
        }
    }

    pub fn jump(self) -> RedHatBoyState<Jumping> {
        RedHatBoyState {
            context: self
                .context
                .set_vertical_velocity(JUMP_SPEED)
                .reset_frame()
                .play_jump_sound(),
            _state: Jumping {},
        }
    }

    pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.set_on(position),
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
pub struct Sliding;

pub enum SlidingEndState {
    Complete(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
}

impl RedHatBoyState<Sliding> {
    pub fn frame_name(&self) -> &str {
        SLIDING_FRAME_NAME
    }

    pub fn update(mut self) -> SlidingEndState {
        self.update_context(SLIDING_FRAMES);

        if self.context.frame >= SLIDING_FRAMES {
            SlidingEndState::Complete(self.stand())
        } else {
            SlidingEndState::Sliding(self)
        }
    }

    pub fn stand(self) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.reset_frame(),
            _state: Running,
        }
    }

    pub fn land_on(self, position: i16) -> RedHatBoyState<Sliding> {
        RedHatBoyState {
            context: self.context.set_on(position),
            _state: Sliding,
        }
    }

    pub fn knock_out(self) -> RedHatBoyState<Falling> {
        RedHatBoyState {
            context: self.context.reset_frame().stop(),
            _state: Falling {},
        }
    }
}

#[derive(Copy, Clone)]
pub struct Jumping;

pub enum JumpingEndState {
    Complete(RedHatBoyState<Running>),
    Jumping(RedHatBoyState<Jumping>),
}

impl RedHatBoyState<Jumping> {
    pub fn frame_name(&self) -> &str {
        JUMPING_FRAME_NAME
    }

    pub fn update(mut self) -> JumpingEndState {
        self.update_context(JUMPING_FRAMES);

        if self.context.position.y >= FLOOR {
            JumpingEndState::Complete(self.land_on(HEIGHT))
        } else {
            JumpingEndState::Jumping(self)
        }
    }

    pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.reset_frame().set_on(position),
            _state: Running,
        }
    }

    pub fn knock_out(self) -> RedHatBoyState<Falling> {
        RedHatBoyState {
            context: self.context.reset_frame().stop(),
            _state: Falling {},
        }
    }
}

#[derive(Copy, Clone)]
pub struct Falling;

pub enum FallingEndState {
    Complete(RedHatBoyState<KnockedOut>),
    Falling(RedHatBoyState<Falling>),
}

impl RedHatBoyState<Falling> {
    pub fn frame_name(&self) -> &str {
        FALLING_FRAME_NAME
    }

    pub fn update(mut self) -> FallingEndState {
        self.update_context(FALLING_FRAMES);

        if self.context.frame >= FALLING_FRAMES {
            FallingEndState::Complete(self.expire())
        } else {
            FallingEndState::Falling(self)
        }
    }

    pub fn expire(self) -> RedHatBoyState<KnockedOut> {
        RedHatBoyState {
            context: self.context,
            _state: KnockedOut,
        }
    }
}

#[derive(Copy, Clone)]
pub struct KnockedOut;

impl RedHatBoyState<KnockedOut> {
    pub fn frame_name(&self) -> &str {
        FALLING_FRAME_NAME
    }
}

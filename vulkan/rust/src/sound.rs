//! Audio events emitted by the game logic and consumed by audio.rs.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sound {
    BlockCollision,
    Goalline,
    StarPickUp,
    GainHeart,
    GhostPlay,
    GhostStop,
    MusicLoop,
    MusicStop,
}

fn baritone_movement(forward: &mut f32, strafe: &mut f32, jump: &mut bool, sneak: &mut bool) {
    use crate::game::baritone as b;
    let (ahead, hop, on) = b::take_move();
    if !on {
        return;
    }
    *forward = ahead;
    *strafe = 0.0;
    *jump = hop;
    *sneak = false;
}

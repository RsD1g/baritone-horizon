fn baritone_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::baritone as b;
    if !game.native_on(b::ID) {
        return;
    }
    let Some(player) = game.player.as_ref() else {
        return;
    };
    let feet = [player.position.x, player.position.y, player.position.z];
    let solid = |at: [i32; 3]| block_solid(game, at);
    let find = |id: &str| nearest(game, feet, id);
    let look = b::tick(feet, &|at| b::feet_ok(at, &solid), &find);
    if look.drive {
        game.yaw = look.yaw;
        game.pitch = look.pitch.clamp(-90.0, 90.0);
    }
}

fn block_solid(game: &crate::game::Game<'_>, at: [i32; 3]) -> bool {
    let name = game.native_block(at);
    if name.contains("water") || name.contains("lava") {
        return true;
    }
    !game.native_block_collision(at).is_empty()
}

fn nearest(game: &crate::game::Game<'_>, feet: [f64; 3], id: &str) -> Option<[i32; 3]> {
    let origin = [feet[0].floor() as i32, feet[1].floor() as i32, feet[2].floor() as i32];
    let mut best: Option<[i32; 3]> = None;
    let mut best_d = i64::MAX;
    for y in -8..=8 {
        for x in -16..=16 {
            for z in -16..=16 {
                let at = [origin[0] + x, origin[1] + y, origin[2] + z];
                if game.native_block(at) != id {
                    continue;
                }
                let d = (x as i64).pow(2) + (y as i64).pow(2) + (z as i64).pow(2);
                if d < best_d {
                    best_d = d;
                    best = Some(at);
                }
            }
        }
    }
    best
}

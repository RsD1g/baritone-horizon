fn baritone_mine(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::baritone as b;
    if !game.native_on(b::ID) {
        return;
    }
    let Some(pos) = b::take_break() else {
        return;
    };
    let Some(player) = game
        .native_server_players()
        .into_iter()
        .find(|player| player.dimension == dimension && player.health > 0.0)
    else {
        return;
    };
    if game.native_block(pos) == "minecraft:air" {
        return;
    }
    game.native_player_break_block(player.uuid, pos, true);
}

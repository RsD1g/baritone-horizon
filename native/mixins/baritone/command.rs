fn baritone_command(
    game: &mut crate::game::Game<'_>,
    _player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::baritone as b;
    if !game.native_on(b::ID) {
        return;
    }
    let Some(lines) = b::handle(command) else {
        return;
    };
    *ran = true;
    for line in lines {
        game.native_feedback(horizon_client::text::Text::literal(line));
    }
}

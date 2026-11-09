fn baritone_register(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    use crate::game::baritone as b;
    if !game.native_on(b::ID) {
        return;
    }
    for usage in b::command_usages() {
        commands.push(horizon_client::mod_api::NativeCommand::new(usage));
    }
}

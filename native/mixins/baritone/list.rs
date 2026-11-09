fn baritone_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::baritone as b;
    mods.push(crate::game::native_hooks::NativeMod {
        id: b::ID.into(),
        title: format!("{} {}", b::NAME, b::VERSION),
        description: b::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

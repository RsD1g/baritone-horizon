fn baritone_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::baritone as b;
    use horizon_client::extras::Corner;
    if !game.native_on(b::ID) || game.native_player().is_none() {
        return;
    }
    if !game.native_corner_shown(Corner::TopLeft) {
        return;
    }
    let lines = b::hud_lines();
    let width = lines
        .iter()
        .map(|line| game.native_text_width(line))
        .fold(0.0_f32, f32::max)
        + 8.0;
    let height = lines.len() as f32 * 10.0 + 4.0;
    let (x, y) = corners.place(Corner::TopLeft, width, height);
    game.native_fill(batch, [x, y, width, height], 0x78101820);
    for (index, line) in lines.iter().enumerate() {
        let color = if index == 0 { 0xFFD27A } else { 0xE8EEF2 };
        game.native_text(
            batch,
            line,
            [x + 4.0, y + 3.0 + index as f32 * 10.0],
            color,
            true,
        );
    }
}

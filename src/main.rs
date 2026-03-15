use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use hanoi_logic::{HanoiGame, Move as HanoiMove, Peg, solve_from_current};

// ── Constants ──────────────────────────────────────────────────────────

const WINDOW_W: f32 = 1200.0;
const WINDOW_H: f32 = 750.0;

const PEG_SPACING: f32 = 350.0;
const PEG_WIDTH: f32 = 12.0;
const PEG_HEIGHT: f32 = 300.0;
const PEG_Y_BASE: f32 = -100.0;

const BASE_WIDTH: f32 = 1100.0;
const BASE_HEIGHT: f32 = 18.0;

const DISK_HEIGHT: f32 = 30.0;
const DISK_GAP: f32 = 2.0;
const MIN_DISK_WIDTH: f32 = 50.0;
const MAX_DISK_WIDTH: f32 = 200.0;

const DISK_Z: f32 = 5.0;
const PEG_Z: f32 = 1.0;
const DRAG_Z: f32 = 20.0;

const DEFAULT_DISKS: u8 = 5;
const MIN_DISKS: u8 = 2;
const MAX_DISKS: u8 = 10;

const AUTO_SOLVE_INTERVAL: f32 = 0.35;

// The "design" world size the game is laid out in. The camera will scale
// so this area always fits in the window, letterboxing if needed.
const WORLD_W: f32 = 1200.0;
const WORLD_H: f32 = 700.0;

// ── Disk appearance ────────────────────────────────────────────────────

fn disk_color(disk: u8, total: u8) -> Color {
    let colors: &[Color] = &[
        Color::from(tailwind::RED_500),
        Color::from(tailwind::ORANGE_500),
        Color::from(tailwind::AMBER_400),
        Color::from(tailwind::LIME_500),
        Color::from(tailwind::EMERALD_500),
        Color::from(tailwind::CYAN_400),
        Color::from(tailwind::BLUE_500),
        Color::from(tailwind::INDIGO_500),
        Color::from(tailwind::PURPLE_500),
        Color::from(tailwind::PINK_500),
    ];
    let idx = if total <= colors.len() as u8 {
        (disk - 1) as usize
    } else {
        ((disk - 1) as f32 / (total - 1).max(1) as f32 * (colors.len() - 1) as f32) as usize
    };
    colors[idx.min(colors.len() - 1)]
}

fn disk_width(disk: u8, total: u8) -> f32 {
    let t = (disk - 1) as f32 / (total - 1).max(1) as f32;
    MIN_DISK_WIDTH + t * (MAX_DISK_WIDTH - MIN_DISK_WIDTH)
}

// ── Resources ──────────────────────────────────────────────────────────

#[derive(Resource)]
struct GameState {
    game: HanoiGame,
}

/// Tracks click-click and keyboard peg selection.
#[derive(Resource, Default)]
struct InputState {
    selected_peg: Option<Peg>,
}

#[derive(Resource, Default)]
struct DragState {
    dragging: Option<DragInfo>,
}

struct DragInfo {
    disk: u8,
    from_peg: Peg,
    entity: Entity,
    offset: Vec2,
}

#[derive(Resource, Default)]
struct AutoSolveState {
    active: bool,
    moves: Vec<HanoiMove>,
    move_index: usize,
    timer: f32,
}

#[derive(Resource)]
struct DiskCountSetting {
    count: u8,
}

impl Default for DiskCountSetting {
    fn default() -> Self {
        Self {
            count: DEFAULT_DISKS,
        }
    }
}

// ── Components ─────────────────────────────────────────────────────────

#[derive(Component)]
struct DiskVisual {
    disk: u8,
}

#[derive(Component)]
struct MoveCounterText;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct DiskCountText;

#[derive(Component)]
struct SelectionIndicator;

// ── Helpers ────────────────────────────────────────────────────────────

fn peg_x(peg: Peg) -> f32 {
    (peg.index() as f32 - 1.0) * PEG_SPACING
}

fn disk_y_on_peg(position_from_bottom: usize) -> f32 {
    PEG_Y_BASE + BASE_HEIGHT / 2.0 + DISK_HEIGHT / 2.0
        + position_from_bottom as f32 * (DISK_HEIGHT + DISK_GAP)
}

fn world_pos_to_peg(x: f32) -> Option<Peg> {
    let half_spacing = PEG_SPACING / 2.0;
    for peg in Peg::ALL {
        if (x - peg_x(peg)).abs() < half_spacing {
            return Some(peg);
        }
    }
    None
}

fn get_world_cursor(
    window: &Window,
    camera: &Camera,
    cam_transform: &GlobalTransform,
) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cp| camera.viewport_to_world_2d(cam_transform, cp).ok())
}

/// Find which disk entity is under the cursor (only top disks are draggable).
fn find_top_disk_at(
    world_pos: Vec2,
    game: &HanoiGame,
    disk_query: &Query<(Entity, &DiskVisual, &Transform)>,
) -> Option<(Entity, u8, Peg, Vec2)> {
    for (entity, disk_vis, transform) in disk_query.iter() {
        let pos = transform.translation;
        let w = disk_width(disk_vis.disk, game.num_disks());
        let half_w = w / 2.0;
        let half_h = DISK_HEIGHT / 2.0;

        if world_pos.x >= pos.x - half_w
            && world_pos.x <= pos.x + half_w
            && world_pos.y >= pos.y - half_h
            && world_pos.y <= pos.y + half_h
        {
            if let Some(peg) = world_pos_to_peg(pos.x) {
                if game.top_disk(peg) == Some(disk_vis.disk) {
                    let offset = Vec2::new(world_pos.x - pos.x, world_pos.y - pos.y);
                    return Some((entity, disk_vis.disk, peg, offset));
                }
            }
        }
    }
    None
}

// ── Main ───────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower of Hanoi".to_string(),
                resolution: (WINDOW_W as u32, WINDOW_H as u32).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameState {
            game: HanoiGame::new(DEFAULT_DISKS),
        })
        .insert_resource(InputState::default())
        .insert_resource(DragState::default())
        .insert_resource(AutoSolveState::default())
        .insert_resource(DiskCountSetting::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                fit_camera,
                keyboard_input,
                mouse_input,
                drag_move,
                auto_solve_system,
            ),
        )
        .add_systems(Update, (update_visuals, update_ui))
        .run();
}

// ── Setup ──────────────────────────────────────────────────────────────

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Base platform
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.35, 0.25, 0.15),
            Vec2::new(BASE_WIDTH, BASE_HEIGHT),
        ),
        Transform::from_xyz(0.0, PEG_Y_BASE, 2.0),
        Pickable::IGNORE,
    ));

    // Pegs
    for peg in Peg::ALL {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.45, 0.32, 0.2),
                Vec2::new(PEG_WIDTH, PEG_HEIGHT),
            ),
            Transform::from_xyz(peg_x(peg), PEG_Y_BASE + PEG_HEIGHT / 2.0, PEG_Z),
            Pickable::IGNORE,
        ));
    }

    // Selection indicator (hidden by default)
    commands.spawn((
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 0.3, 0.25),
            Vec2::new(MAX_DISK_WIDTH + 40.0, PEG_HEIGHT + DISK_HEIGHT * 2.0),
        ),
        Transform::from_xyz(0.0, PEG_Y_BASE + PEG_HEIGHT / 2.0, 0.5),
        Visibility::Hidden,
        SelectionIndicator,
        Pickable::IGNORE,
    ));

    spawn_disks(&mut commands, DEFAULT_DISKS);

    // ── UI ──

    // Top-left: move counter
    commands.spawn((
        Text::new(format!(
            "Moves: 0 / {} optimal",
            (1u32 << DEFAULT_DISKS) - 1
        )),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(16.0),
            left: px(16.0),
            ..default()
        },
        MoveCounterText,
    ));

    // Top-center: status
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::from(tailwind::YELLOW_300)),
        Node {
            position_type: PositionType::Absolute,
            top: px(16.0),
            right: px(16.0),
            ..default()
        },
        StatusText,
    ));

    // Bottom: controls help
    commands.spawn((
        Text::new(
            "[1/2/3] Select peg  |  Click peg  |  Drag disk  |  \
             [+/-] Disks  |  [S] Solve  |  [R] Reset  |  [Esc] Cancel",
        ),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12.0),
            left: px(0.0),
            right: px(0.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
    ));

    // Bottom-right: disk count
    commands.spawn((
        Text::new(format!("Disks: {}", DEFAULT_DISKS)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(40.0),
            right: px(16.0),
            ..default()
        },
        DiskCountText,
    ));

    // Peg number labels (world-space so they stay aligned on resize)
    for (i, label) in ["1", "2", "3"].iter().enumerate() {
        let peg = Peg::from_index(i).unwrap();
        commands.spawn((
            Text2d::new(*label),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 0.7)),
            Transform::from_xyz(peg_x(peg), PEG_Y_BASE - 45.0, 10.0),
            Pickable::IGNORE,
        ));
    }
}

fn spawn_disks(commands: &mut Commands, num_disks: u8) {
    for disk in 1..=num_disks {
        let stack_pos = (num_disks - disk) as usize;
        let w = disk_width(disk, num_disks);
        let color = disk_color(disk, num_disks);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(w, DISK_HEIGHT)),
            Transform::from_xyz(peg_x(Peg::Left), disk_y_on_peg(stack_pos), DISK_Z),
            DiskVisual { disk },
        ));
    }
}

fn despawn_disks(commands: &mut Commands, disk_query: &Query<Entity, With<DiskVisual>>) {
    for entity in disk_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn reset_game(
    game_state: &mut ResMut<GameState>,
    auto_solve: &mut ResMut<AutoSolveState>,
    input_state: &mut ResMut<InputState>,
    drag_state: &mut ResMut<DragState>,
    commands: &mut Commands,
    disk_query: &Query<Entity, With<DiskVisual>>,
    num_disks: u8,
) {
    game_state.game.reset_with(num_disks);
    auto_solve.active = false;
    input_state.selected_peg = None;
    drag_state.dragging = None;
    despawn_disks(commands, disk_query);
    spawn_disks(commands, num_disks);
}

// ── Camera scaling ─────────────────────────────────────────────────────

fn fit_camera(
    window: Single<&Window>,
    mut camera_q: Query<&mut Projection, With<Camera2d>>,
) {
    let Ok(mut projection) = camera_q.single_mut() else {
        return;
    };
    if let Projection::Orthographic(ref mut ortho) = *projection {
        let win_w = window.width();
        let win_h = window.height();
        let scale_x = WORLD_W / win_w;
        let scale_y = WORLD_H / win_h;
        ortho.scale = scale_x.max(scale_y);
    }
}

// ── Keyboard input ─────────────────────────────────────────────────────

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut input_state: ResMut<InputState>,
    mut auto_solve: ResMut<AutoSolveState>,
    mut disk_count: ResMut<DiskCountSetting>,
    mut commands: Commands,
    disk_query: Query<Entity, With<DiskVisual>>,
    mut drag_state: ResMut<DragState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        input_state.selected_peg = None;
        auto_solve.active = false;
        drag_state.dragging = None;
        return;
    }

    if keys.just_pressed(KeyCode::KeyS) && !game_state.game.is_solved() {
        let moves = solve_from_current(&game_state.game);
        *auto_solve = AutoSolveState {
            active: true,
            moves,
            move_index: 0,
            timer: 0.0,
        };
        input_state.selected_peg = None;
        return;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        let n = game_state.game.num_disks();
        reset_game(
            &mut game_state,
            &mut auto_solve,
            &mut input_state,
            &mut drag_state,
            &mut commands,
            &disk_query,
            n,
        );
        return;
    }

    // Adjust disk count
    let new_count =
        if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
            Some((disk_count.count + 1).min(MAX_DISKS))
        } else if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
            Some(disk_count.count.saturating_sub(1).max(MIN_DISKS))
        } else {
            None
        };

    if let Some(n) = new_count {
        disk_count.count = n;
        reset_game(
            &mut game_state,
            &mut auto_solve,
            &mut input_state,
            &mut drag_state,
            &mut commands,
            &disk_query,
            n,
        );
        return;
    }

    // Peg selection via 1/2/3 keys
    if auto_solve.active {
        return;
    }
    let peg = if keys.just_pressed(KeyCode::Digit1) {
        Some(Peg::Left)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(Peg::Middle)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(Peg::Right)
    } else {
        None
    };
    if let Some(peg) = peg {
        do_peg_selection(&mut game_state, &mut input_state, peg);
    }
}

fn do_peg_selection(
    game_state: &mut ResMut<GameState>,
    input_state: &mut ResMut<InputState>,
    peg: Peg,
) {
    if let Some(from_peg) = input_state.selected_peg {
        if from_peg == peg {
            input_state.selected_peg = None;
        } else {
            let _ = game_state.game.make_move(HanoiMove {
                from: from_peg,
                to: peg,
            });
            input_state.selected_peg = None;
        }
    } else if game_state.game.top_disk(peg).is_some() {
        input_state.selected_peg = Some(peg);
    }
}

// ── Mouse input (unified click + drag start + drag end) ────────────────

fn mouse_input(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_q: Single<(&Camera, &GlobalTransform)>,
    mut game_state: ResMut<GameState>,
    mut input_state: ResMut<InputState>,
    mut drag_state: ResMut<DragState>,
    auto_solve: Res<AutoSolveState>,
    disk_query: Query<(Entity, &DiskVisual, &Transform)>,
) {
    if auto_solve.active {
        return;
    }

    let (camera, cam_transform) = *camera_q;

    // ── Mouse button released: finish drag ──
    if mouse.just_released(MouseButton::Left) {
        if let Some(info) = drag_state.dragging.take() {
            let target_peg = get_world_cursor(&window, camera, cam_transform)
                .and_then(|wp| world_pos_to_peg(wp.x));
            if let Some(to_peg) = target_peg {
                if to_peg != info.from_peg {
                    let _ = game_state.game.make_move(HanoiMove {
                        from: info.from_peg,
                        to: to_peg,
                    });
                }
            }
            // update_visuals will snap the disk to its correct position
        }
        return;
    }

    // ── Mouse button pressed ──
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = get_world_cursor(&window, camera, cam_transform) else {
        return;
    };

    // If a peg is already selected (click-click mode), second click completes the move
    if input_state.selected_peg.is_some() {
        if let Some(peg) = world_pos_to_peg(world_pos.x) {
            do_peg_selection(&mut game_state, &mut input_state, peg);
        }
        return;
    }

    // Try to start a drag on a top disk
    if let Some((entity, disk, from_peg, offset)) =
        find_top_disk_at(world_pos, &game_state.game, &disk_query)
    {
        drag_state.dragging = Some(DragInfo {
            disk,
            from_peg,
            entity,
            offset,
        });
        return;
    }

    // Click on a peg area to select it (click-click mode)
    if let Some(peg) = world_pos_to_peg(world_pos.x) {
        if game_state.game.top_disk(peg).is_some() {
            input_state.selected_peg = Some(peg);
        }
    }
}

// ── Drag movement ──────────────────────────────────────────────────────

fn drag_move(
    window: Single<&Window>,
    camera_q: Single<(&Camera, &GlobalTransform)>,
    drag_state: Res<DragState>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(ref info) = drag_state.dragging else {
        return;
    };
    let (camera, cam_transform) = *camera_q;
    let Some(world_pos) = get_world_cursor(&window, camera, cam_transform) else {
        return;
    };
    if let Ok(mut transform) = transforms.get_mut(info.entity) {
        transform.translation.x = world_pos.x - info.offset.x;
        transform.translation.y = world_pos.y - info.offset.y;
        transform.translation.z = DRAG_Z;
    }
}

// ── Auto-solve ─────────────────────────────────────────────────────────

fn auto_solve_system(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut auto_solve: ResMut<AutoSolveState>,
) {
    if !auto_solve.active {
        return;
    }
    auto_solve.timer += time.delta_secs();
    if auto_solve.timer >= AUTO_SOLVE_INTERVAL {
        auto_solve.timer = 0.0;
        if auto_solve.move_index < auto_solve.moves.len() {
            let m = auto_solve.moves[auto_solve.move_index];
            let _ = game_state.game.make_move(m);
            auto_solve.move_index += 1;
        } else {
            auto_solve.active = false;
        }
    }
}

// ── Visual sync ────────────────────────────────────────────────────────

fn update_visuals(
    game_state: Res<GameState>,
    drag_state: Res<DragState>,
    input_state: Res<InputState>,
    mut disk_query: Query<(&DiskVisual, &mut Transform, &mut Sprite)>,
    mut sel_query: Query<
        (&mut Transform, &mut Visibility),
        (With<SelectionIndicator>, Without<DiskVisual>),
    >,
) {
    let game = &game_state.game;

    for (disk_vis, mut transform, mut sprite) in disk_query.iter_mut() {
        // Skip the disk being dragged
        if let Some(ref info) = drag_state.dragging {
            if info.disk == disk_vis.disk {
                continue;
            }
        }

        for peg in Peg::ALL {
            let disks = game.disks_on(peg);
            if let Some(pos) = disks.iter().position(|&d| d == disk_vis.disk) {
                transform.translation.x = peg_x(peg);
                transform.translation.y = disk_y_on_peg(pos);
                transform.translation.z = DISK_Z;

                // Lift top disk of selected peg
                if input_state.selected_peg == Some(peg) && pos == disks.len() - 1 {
                    transform.translation.y += 15.0;
                    transform.translation.z = DISK_Z + 1.0;
                }
                break;
            }
        }

        let w = disk_width(disk_vis.disk, game.num_disks());
        sprite.custom_size = Some(Vec2::new(w, DISK_HEIGHT));
        sprite.color = disk_color(disk_vis.disk, game.num_disks());
    }

    // Selection indicator
    if let Ok((mut transform, mut visibility)) = sel_query.single_mut() {
        if let Some(peg) = input_state.selected_peg {
            *visibility = Visibility::Visible;
            transform.translation.x = peg_x(peg);
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

// ── UI sync ────────────────────────────────────────────────────────────

fn update_ui(
    game_state: Res<GameState>,
    auto_solve: Res<AutoSolveState>,
    disk_count: Res<DiskCountSetting>,
    mut counter_q: Query<
        &mut Text,
        (
            With<MoveCounterText>,
            Without<StatusText>,
            Without<DiskCountText>,
        ),
    >,
    mut status_q: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<MoveCounterText>,
            Without<DiskCountText>,
        ),
    >,
    mut disk_count_q: Query<
        &mut Text,
        (
            With<DiskCountText>,
            Without<MoveCounterText>,
            Without<StatusText>,
        ),
    >,
) {
    let game = &game_state.game;

    if let Ok(mut text) = counter_q.single_mut() {
        **text = format!(
            "Moves: {} / {} optimal",
            game.move_count(),
            game.minimum_moves()
        );
    }

    if let Ok(mut text) = status_q.single_mut() {
        if game.is_solved() {
            let moves = game.move_count();
            let optimal = game.minimum_moves();
            if moves == optimal {
                **text = "Solved! Perfect score!".to_string();
            } else {
                **text = format!("Solved in {} moves! (optimal: {})", moves, optimal);
            }
        } else if auto_solve.active {
            **text = "Auto-solving...".to_string();
        } else {
            **text = String::new();
        }
    }

    if let Ok(mut text) = disk_count_q.single_mut() {
        **text = format!("Disks: {}", disk_count.count);
    }
}

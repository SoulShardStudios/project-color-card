use crate::assets::LoadState;
use crate::cards::Card;
use crate::game_state::CardStats;
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub enum CustomCursor {
    #[default]
    Default,
    Card {
        card: AssetId<Card>,
        stats: CardStats,
    },
}

fn spawn_custom_cursor(mut commands: Commands, mut window: Query<&mut Window>) {
    commands
        .spawn(ImageBundle {
            style: Style {
                width: Val::Px(72.0),
                aspect_ratio: Some(72.0 / 102.0),
                ..default()
            },
            ..default()
        })
        .insert(CustomCursor::Default);
    match window.get_single_mut() {
        Ok(mut x) => x.cursor.visible = false,
        _ => {}
    }
}

fn move_custom_cursor(
    mut cursor_evr: EventReader<CursorMoved>,
    mut cursor_query: Query<&mut Style, With<CustomCursor>>,
) {
    for ev in cursor_evr.read() {
        match cursor_query.get_single_mut() {
            Ok(mut cursor) => {
                cursor.left = Val::Px(ev.position.x);
                cursor.top = Val::Px(ev.position.y);
            }
            _ => {}
        }
    }
}

fn manage_custom_cursor_asset(
    mut custom_cursor_query: Query<(&mut CustomCursor, &mut UiImage, &mut Style)>,
    cards: Res<Assets<Card>>,
    assets: Res<AssetServer>,
) {
    let (cursor, mut image, mut style) = match custom_cursor_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    match cursor.to_owned() {
        CustomCursor::Card {
            card,
            stats: _cursor_stats,
        } => {
            match cards.get(card) {
                Some(x) => image.texture = x.image_handle.clone(),
                None => {}
            };
            (*style).width = Val::Px(72.0);
            (*style).aspect_ratio = Some(72.0 / 102.0);
        }
        CustomCursor::Default => {
            image.texture = assets.load("ui/Cursor.png");
            (*style).width = Val::Px(21.0);
            (*style).aspect_ratio = Some(21.0 / 27.0);
        }
    }
}

pub struct CustomCursorPlugin;

impl Plugin for CustomCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LoadState::Loaded), spawn_custom_cursor)
            .add_systems(Update, (move_custom_cursor, manage_custom_cursor_asset));
    }
}

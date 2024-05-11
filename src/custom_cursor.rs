use crate::assets::LoadState;
use crate::cards::Card;
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub enum CustomCursor {
    #[default]
    Default,
    Card(AssetId<Card>),
}

impl CustomCursor {
    pub fn get_current_card(&self) -> Option<AssetId<Card>> {
        match self {
            CustomCursor::Card(x) => Some(*x),
            _ => None,
        }
    }
}

fn spawn_custom_cursor(mut commands: Commands) {
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
    mut custom_cursor_query: Query<(&mut CustomCursor, &mut UiImage, &mut Visibility)>,
    mut window: Query<&mut Window>,
    cards: Res<Assets<Card>>,
) {
    let (cursor, mut image, mut visibility) = match custom_cursor_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    match cursor.to_owned() {
        CustomCursor::Card(x) => {
            match cards.get(x) {
                Some(x) => image.texture = x.image_handle.clone(),
                None => {}
            };
            *visibility = Visibility::Visible;
            match window.get_single_mut() {
                Ok(mut x) => x.cursor.visible = false,
                _ => {}
            };
        }
        CustomCursor::Default => {
            image.texture = Handle::default();
            *visibility = Visibility::Hidden;
            match window.get_single_mut() {
                Ok(mut x) => x.cursor.visible = true,
                _ => {}
            };
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

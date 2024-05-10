use crate::assets::LoadState;
use crate::cards::Card;
use bevy::prelude::*;

#[derive(Default, Clone)]
enum CustomCursorData {
    #[default]
    Default,
    Card(AssetId<Card>),
}

#[derive(Component)]
pub struct CustomCursor {
    current_cursor: CustomCursorData,
}

impl CustomCursor {
    pub fn new() -> Self {
        Self {
            current_cursor: CustomCursorData::Default,
        }
    }
    pub fn set_default(&mut self) {
        self.current_cursor = CustomCursorData::Default;
    }
    pub fn set_card(&mut self, card: AssetId<Card>) {
        self.current_cursor = CustomCursorData::Card(card);
    }
    pub fn get_current_card(&self) -> Option<AssetId<Card>> {
        match self.current_cursor {
            CustomCursorData::Card(x) => Some(x),
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
        .insert(CustomCursor {
            current_cursor: CustomCursorData::Default,
        });
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
    mut custom_cursor_query: Query<(&mut CustomCursor, &mut UiImage)>,
    mut window: Query<&mut Window>,
    cards: Res<Assets<Card>>,
) {
    let (cursor, mut image) = match custom_cursor_query.get_single_mut() {
        Ok(x) => x,
        _ => {
            return;
        }
    };
    match cursor.current_cursor {
        CustomCursorData::Card(x) => {
            match cards.get(x) {
                Some(x) => image.texture = x.image_handle.clone(),
                None => {}
            }
            match window.get_single_mut() {
                Ok(mut x) => x.cursor.visible = false,
                _ => {}
            };
        }
        CustomCursorData::Default => {
            image.texture = Handle::default();
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

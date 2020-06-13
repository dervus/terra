

pub struct Note {
    pub note_id: i32,
    pub account: AccountRef,
    pub contents: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

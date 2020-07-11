use warp::Reply;
use crate::init::CCtxRef;
use crate::view::{self, Page, SpecialPage};
use crate::db;
use super::FilterResult;

pub fn new_page(cgn: CCtxRef, session: super::session::Session) -> impl Reply {
    edit_page(cgn, session, &Default::default())
}

pub fn edit_page(cgn: CCtxRef, session: super::session::Session, form: &view::pc::CharacterForm) -> impl Reply {
    view::pc::form(&cgn.data, form).render(Some(&session))
}

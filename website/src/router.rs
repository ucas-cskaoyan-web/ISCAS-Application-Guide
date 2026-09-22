use leptos::prelude::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

use crate::{
    components::layout::AppLayout,
    pages::{
        about::AboutPage, article_view::ArticlePage, articles_list::ArticlesListPage,
        error_pages::NotFoundPage, home::HomePage,
    },
};

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| {
                view! { <NotFoundPage /> }
            }>
                <ParentRoute path=path!("") view=AppLayout>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("articles") view=ArticlesListPage />
                    <Route path=path!("about") view=AboutPage />
                    <Route path=path!("articles/about") view=AboutPage />
                    <Route path=path!("articles/:id") view=ArticlePage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

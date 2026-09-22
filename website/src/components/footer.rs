use leptos::{component, prelude::*};
use leptos_router::components::A;

use crate::app::{TranslationContext, SITE_CONFIGURATION};

#[component]
pub fn Footer() -> impl IntoView {
    let site_config = SITE_CONFIGURATION
        .get()
        .expect("Site configuration should be loaded by AppLayout");
    let translator = expect_context::<TranslationContext>();
    let copyright_holder = site_config.copyright.holder.clone();
    let copyright_notice = site_config.copyright.notice.clone();
    let license = site_config.copyright.license.clone();
    let license_url = site_config.copyright.license_url.clone();

    view! {
        <footer class="footer">
            <div class="footer-content">
                <p class="footer-copyright">
                    {format!("© {} {}", site_config.copyright_year, copyright_holder)}
                    <span aria-hidden="true">" · "</span>
                    <a href=license_url target="_blank" rel="noopener noreferrer">{license}</a>
                </p>
                <p class="footer-powered-by">
                    {format!("{} ", translator.translate("Powered by"))}
                    <a href="https://github.com/ucas-cskaoyan-web/ISCAS-Application-Guide">
                        "ISCAS Application Guide"
                    </a>
                </p>
                <p class="footer-legal">
                    {copyright_notice}
                    <span aria-hidden="true">" · "</span>
                    <A href="/articles/disclaimer">{translator.translate("Disclaimer")}</A>
                </p>
            </div>
        </footer>
    }
}

use dioxus::prelude::*;

/// 서비스 제목. 화면과 문서에서 같은 문구를 쓴다.
const SERVICE_TITLE: &str = "숫자형 단축 URL";

fn main() {
    // 서버 빌드: Dioxus 라우터에 API 라우터를 합쳐 띄운다.
    #[cfg(feature = "server")]
    dioxus::serve(
        || async move { Ok(dioxus::server::router(app).merge(short_url::api::api_router())) },
    );

    // 브라우저 빌드: 화면만 띄운다.
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

/// 빈 생성 화면. 이후 티켓에서 입력 폼을 채운다.
fn app() -> Element {
    rsx! {
        main {
            h1 { "{SERVICE_TITLE}" }
        }
    }
}

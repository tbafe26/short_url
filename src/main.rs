use dioxus::prelude::*;
use short_url::dto::LinkResponse;

/// 서비스 제목. 화면과 문서에서 같은 문구를 쓴다.
const SERVICE_TITLE: &str = "숫자형 단축 URL";

fn main() {
    // 서버 빌드: Dioxus 라우터에 API 라우터를 합쳐 띄운다.
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        let state = server_state().await?;
        Ok(dioxus::server::router(app).merge(short_url::api::api_router(state)))
    });

    // 브라우저 빌드: 화면만 띄운다.
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

/// 환경변수에서 설정을 읽어 DB에 연결하고 마이그레이션을 적용한다.
#[cfg(feature = "server")]
async fn server_state() -> anyhow::Result<short_url::api::AppState> {
    use anyhow::Context;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL이 필요하다")?;
    let public_base_url = std::env::var("PUBLIC_BASE_URL").context("PUBLIC_BASE_URL이 필요하다")?;

    // 유휴 연결이 남아 있으면 Railway 절전이 되지 않으므로 빨리 닫는다.
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .context("DB에 연결하지 못했다")?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("마이그레이션 실패")?;
    Ok(short_url::api::AppState::new(pool, public_base_url))
}

/// 생성 API를 호출한다. 같은 출처의 `/api/links`로 요청한다.
#[cfg(feature = "web")]
async fn request_link(url: String) -> Result<LinkResponse, String> {
    use short_url::dto::CreateLinkRequest;

    let req = gloo_net::http::Request::post("/api/links")
        .json(&CreateLinkRequest { url })
        .map_err(|e| e.to_string())?;
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == 201 {
        res.json::<LinkResponse>().await.map_err(|e| e.to_string())
    } else if res.status() == 400 {
        // 서버가 보낸 검증 오류 메시지를 그대로 보여 준다.
        let body = res.json::<serde_json::Value>().await.ok();
        let message = body
            .as_ref()
            .and_then(|v| v["error"].as_str())
            .unwrap_or("올바르지 않은 요청입니다.");
        Err(message.to_string())
    } else {
        Err(format!("생성하지 못했다 (상태 {})", res.status()))
    }
}

/// 브라우저가 아닌 빌드(서버 렌더링)에서는 호출되지 않는다.
#[cfg(not(feature = "web"))]
async fn request_link(_url: String) -> Result<LinkResponse, String> {
    Err("브라우저에서만 생성할 수 있다".to_string())
}

/// 생성 화면: 원본 URL을 입력하고 단축 주소를 받는다.
fn app() -> Element {
    let mut input = use_signal(String::new);
    let mut result = use_signal(|| None::<LinkResponse>);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    let submit = move |_| {
        let url = input.read().clone();
        spawn(async move {
            busy.set(true);
            error.set(None);
            match request_link(url).await {
                Ok(link) => result.set(Some(link)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    rsx! {
        main {
            h1 { "{SERVICE_TITLE}" }
            input {
                r#type: "text",
                placeholder: "https://example.com/...",
                value: "{input}",
                oninput: move |e| input.set(e.value()),
            }
            button { disabled: *busy.read(), onclick: submit,
                if *busy.read() { "생성 중..." } else { "생성" }
            }
            if let Some(message) = error.read().clone() {
                p { role: "alert", "{message}" }
            }
            if let Some(link) = result.read().clone() {
                p {
                    "단축 주소: "
                    a { href: "{link.short_url}", "{link.short_url}" }
                }
            }
        }
    }
}

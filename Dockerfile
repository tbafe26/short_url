# Railway 백엔드 이미지 (ADR-0003: Railway가 이 파일로 직접 빌드한다)
#
# 1단계: Dioxus CLI(dx)로 서버 실행 파일과 웹 번들(public/)을 release로 빌드한다.
# 2단계: 실행 파일과 public/만 작은 이미지에 복사한다.
#
# cargo-chef는 쓰지 않는다. dx가 자체 프로필(server-release, wasm-release)로 빌드해
# cargo-chef의 의존성 사전 빌드 결과를 재사용하지 못하기 때문이다.

# ---- 1단계: 빌드 ----
FROM rust:1-bookworm AS builder

# dx는 소스 빌드가 오래 걸리므로 cargo-binstall로 미리 빌드된 바이너리를 받는다.
# 라이브러리 버전(Cargo.toml의 dioxus =0.7.10)과 같은 버전으로 고정한다.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall dioxus-cli@0.7.10 --no-confirm \
    && rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
# `sqlx::migrate!`가 컴파일 시점에 마이그레이션을 실행 파일에 넣는다.
COPY migrations ./migrations

RUN dx build --release

# ---- 2단계: 실행 ----
FROM debian:bookworm-slim AS runtime

# DB 접속(rustls)에 쓸 루트 인증서
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# 서버 실행 파일은 같은 폴더의 public/(웹 번들)을 찾으므로 폴더째 복사한다.
COPY --from=builder /app/target/dx/short_url/release/web/ /app/

# Dioxus 서버는 IP와 PORT 환경변수로 바인드한다. Railway가 PORT를 주입한다.
ENV IP=0.0.0.0
ENV PORT=8080
EXPOSE 8080

CMD ["/app/server"]

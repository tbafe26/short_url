# 설치 목록과 정리 방법

이 프로젝트를 위해 새로 설치하거나 만든 것의 목록이다. 프로젝트가 끝나면 아래 순서대로 지운다. 새 도구·컨테이너·클라우드 리소스를 추가하면 이 문서에 함께 적는다.

## 원래 있던 것 (지우지 않음)

Node.js, Git, Docker Desktop, winget, `C:\Program Files (x86)\Microsoft Visual Studio`의 Visual Studio Installer.

## 1. 클라우드 리소스 (채점이 끝난 뒤)

| 대상 | 정리 방법 |
| --- | --- |
| GitHub 저장소 `tbafe26/short_url` (공개) | 제출·채점에 필요하므로 남겨 둔다. 지우려면 GitHub 저장소 Settings → Danger Zone → Delete this repository |
| Railway 프로젝트 (백엔드 + Postgres) | Railway 대시보드 → 프로젝트 → Settings → Delete Project |
| Cloudflare Worker | Cloudflare 대시보드 → Workers & Pages → 해당 Worker → Settings → Delete |
| wrangler 로그인 정보 | `npx wrangler logout` |
| Railway CLI 로그인 정보 (설치했다면) | `railway logout` |

카드를 등록하지 않았으므로 지우지 않아도 과금되지 않는다 (ADR-0003).

## 2. Docker

| 대상 | 정리 명령 |
| --- | --- |
| 로컬 테스트 DB 컨테이너 `short-url-pg` | `docker rm -f short-url-pg` |
| `postgres:17` 이미지 | `docker rmi postgres:17` |
| 백엔드 이미지 (`docker build`로 만든 경우) | `docker images`로 이름 확인 후 `docker rmi <이미지>` |
| 빌드 캐시 | `docker builder prune` |

## 3. 프로젝트 폴더

| 대상 | 정리 방법 |
| --- | --- |
| 에이전트 워크트리 (`.claude/worktrees/`) | `git worktree list`로 확인 후 `git worktree remove --force <경로>` |
| Rust 빌드 산출물 (`target/`, 수 GB) | 프로젝트 폴더에서 `cargo clean` |
| 프로젝트 전체 | `short_url` 폴더 삭제 |

## 4. 개발 도구

| 대상 | 위치·크기 | 정리 명령 |
| --- | --- | --- |
| Rust 툴체인 (stable MSVC + `wasm32-unknown-unknown`) | `~/.rustup`, `~/.cargo` | `rustup self uninstall` (PATH 등록도 제거) |
| dioxus-cli 0.7.10 (`dx`) | `~/.cargo/bin/dx.exe` | 위 `rustup self uninstall`에 포함 |
| dioxus-cli 빌드 캐시 | `~/.cargo/dx-build` | 위 `rustup self uninstall`에 포함 |
| VS 2022 Build Tools (C++ 워크로드) | 약 3~6GB | 관리자 PowerShell에서 `winget uninstall --id Microsoft.VisualStudio.2022.BuildTools` |
| 실패한 cargo 설치 잔여물 | `%TEMP%\cargo-install*` | PowerShell에서 `Remove-Item -Recurse -Force "$env:TEMP\cargo-install*"` |
| wrangler npx 캐시 | `%LOCALAPPDATA%\npm-cache\_npx` | `npm cache clean --force` (다른 프로젝트의 npx 캐시도 함께 지워짐) |

## 정리 확인

```bash
rustup --version
```

```bash
docker ps -a --filter name=short-url-pg
```

```bash
winget list --id Microsoft.VisualStudio.2022.BuildTools
```

첫 명령은 "찾을 수 없음", 두 번째는 빈 목록, 세 번째는 "설치된 패키지 없음"이면 정리가 끝난 것이다.

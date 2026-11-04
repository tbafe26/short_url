# 과금이 발생하지 않는 범위에서만 운영한다

이 서비스는 과제 제출용이라 비용이 0원이어야 한다. Railway에는 카드를 등록하지 않고 Trial/Free 크레딧 안에서만 운영하며, 크레딧이 떨어지면 서비스가 멈추는 것을 감수한다. 크레딧을 아끼려고 백엔드는 Railway 절전(App Sleeping)을 켜고 DB는 상시 켜 두며, `sqlx` 풀은 최대 3개로 둔다. 도메인은 사지 않고 Cloudflare의 무료 `workers.dev` 서브도메인을 쓴다.

## Considered Options

- **Neon·Supabase 같은 외부 무료 DB**: 과제가 "Railway에서 운영하는 DB"를 요구하므로 제외했다.
- **GitHub Actions에서 이미지 빌드 후 Railway는 실행만**: Railway 빌드가 메모리·시간 부족으로 실패할 때 전환할 대안으로 남겨 둔다. 처음에는 Railway가 멀티스테이지 `Dockerfile`(cargo-chef 캐시)로 직접 빌드한다.

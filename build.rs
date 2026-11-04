fn main() {
    // 마이그레이션이 바뀌면 `sqlx::migrate!`가 다시 컴파일되도록 한다.
    println!("cargo:rerun-if-changed=migrations");
}

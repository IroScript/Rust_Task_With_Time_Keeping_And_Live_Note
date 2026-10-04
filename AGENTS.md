# AGENTS.md - RUST TASK GOVERNANCE

All AGY agents operating within this directory must strictly adhere to the following rules.

---

### Category XV: Rust Task APK Local High-Speed Delivery Standard (77)
77. `SETTING_77_MANDATORY_LOCAL_APK_HIGH_SPEED_DOWNLOAD_LINK`: Whenever providing APK files, downloads, or build artifacts for the Rust Task project group (`AGY · Rust Task (rust)`), or upon any APK generation/verification, the agent MUST ALWAYS provide the local high-speed Cloudflare tunnel download link (`https://.../api/raw?path=.../output_apk/app-release.apk&download=1`) served directly from the local Azure VM filesystem. Providing GitHub raw or blob links as primary download source is strictly prohibited because local server downloads offer significantly higher transfer speeds (25+ MB/s) and zero CDN rate limiting. GitHub Actions and repo commits remain strictly for automated build logging and CI synchronization.

---

### Category XVII: Rust Task Old APK Purge Mandate Before Build (79)
79. `SETTING_79_MANDATORY_OLD_APK_PURGE_BEFORE_BUILD`: Whenever initiating, executing, or preparing an APK build for the Rust Task project group (`AGY · Rust Task (rust)`), the agent MUST ALWAYS delete and completely purge all pre-existing, stale, or old APK files (`output_apk/*.apk`, `mobile/build/**/*.apk`, `mobile/android/build/**/*.apk`) from the workspace BEFORE running the build. Retaining stale APK artifacts or distributing outdated builds when a new build is requested is strictly prohibited. The workspace output paths must be verified clean of previous APK files prior to every build cycle.

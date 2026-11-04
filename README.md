# AppChime

앱이 **켜지거나 꺼질 때** 지정한 소리를 재생하는 메뉴 막대(트레이) 앱입니다. macOS · Windows 지원.

- 앱마다 "켜질 때 / 꺼질 때" 소리를 따로 켜고 끔
- 소리 파일 여러 개 + 재생 방식: 고정 / 랜덤(연속 중복 없음) / 순서대로
- 앱별 볼륨, 미리듣기, 파일 끌어다 놓기, 순서 바꾸기
- 메뉴 막대에서 전체 소리 켜기/끄기, 원하면 전역 단축키 지정
- 중복 재생 방지, 로그인 시 자동 실행, 다크 모드, 한국어·영어·일본어

---

## 개발 환경 준비 (맥 기준, 처음 한 번)

1. **Xcode 명령줄 도구**: 터미널에서 `xcode-select --install`
2. **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **Node.js 20 이상**: https://nodejs.org 에서 LTS 설치
4. VS Code에서 이 폴더를 열면 추천 확장(Tauri, rust-analyzer)을 설치하라고 뜹니다. 설치하세요.

윈도우에서 개발할 때는 Rust와 Node 외에 "Microsoft C++ Build Tools"가 필요합니다
(https://tauri.app/start/prerequisites/ 참고).

## 실행해 보기

```bash
npm install          # 처음 한 번
npm run app:dev      # 개발 모드로 실행 (코드 저장하면 화면이 바로 바뀜)
```

- 첫 실행은 Rust 라이브러리를 받고 컴파일하느라 몇 분 걸립니다. 다음부터는 빠릅니다.
- 실행하면 메뉴 막대에 스피커 아이콘이 생기고, 등록한 앱이 없으면 설정 창이 열립니다.
- 화면(React)만 고칠 땐 저장하면 바로 반영되고, Rust 코드를 고치면 자동으로 다시 빌드·재시작됩니다.
- 개발 모드에서는 "로그인 시 자동 실행"이 실제로 등록되지 않습니다 (개발용 파일이 등록되는 것을 막기 위해).

### 테스트

```bash
npm run typecheck                                  # 화면 타입 검사
cd crates/app-watch && cargo test                  # 앱 감지 모듈
cd src-tauri && cargo test                         # 본체 (설정·규칙·소리 파일)
```

## 설치 파일 만들기

**내 컴퓨터에서**: `npm run app:build` → `src-tauri/target/release/bundle/` 에 맥은 `.dmg`가 생깁니다.

**GitHub에서 맥·윈도우 둘 다**: 태그를 올리면 자동으로 만들어서 Releases에 초안으로 올립니다.

```bash
git tag v0.1.0
git push origin v0.1.0
```

저장소의 **Actions** 탭에서 진행 상황을 볼 수 있고, 끝나면 **Releases**에 초안이 생깁니다.
확인 후 *Publish release* 를 누르면 공개됩니다. 버전을 올릴 때는 `package.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` 의 `version`을 같이 바꾸세요.

> 서명하지 않은 앱은 처음 열 때 경고가 뜹니다.
> 맥: 앱을 우클릭 → 열기 (또는 시스템 설정 › 개인정보 보호 및 보안 › "그래도 열기")
> 윈도우: "추가 정보" → "실행"
> 정식 배포 시에는 Apple Developer 서명·공증을 권장합니다 (`release.yml` 상단 설명 참고).

---

## 폴더 구조

```
appchime/
├─ src/                        화면 (React + TypeScript)
│  ├─ App.tsx                  전체 레이아웃
│  ├─ features/
│  │  ├─ rules/                메인 창: 앱 목록, 상세, 켜질/꺼질 때 카드, 소리 목록, 볼륨
│  │  ├─ addApp/               앱 추가 창
│  │  └─ settings/             일반 설정 창, 단축키 입력
│  ├─ components/              공용 부품: Switch, Button, Dialog, SegmentedControl, Icon …
│  ├─ store/appStore.ts        화면 상태 + 백엔드 호출 (모든 동작이 여기 모여 있음)
│  ├─ lib/                     타입, 백엔드 호출 목록(api.ts), 단축키·형식 도우미
│  ├─ i18n/                    문구 (ko.ts 기준, en.ts, ja.ts)
│  ├─ styles/tokens.css        색·간격·글꼴 (라이트/다크)
│  └─ theme.ts                 다크 모드 적용
│
├─ src-tauri/                  본체 (Rust)
│  ├─ src/
│  │  ├─ lib.rs                시작 순서, 플러그인, 구조 그림
│  │  ├─ commands.rs           화면에서 부르는 명령
│  │  ├─ state.rs              설정 보관 + 변경 시 저장·알림·메뉴 갱신
│  │  ├─ engine.rs             "어떤 소리를 틀지" 결정 (고정/랜덤/순서, 중복 방지)
│  │  ├─ audio.rs              재생 스레드
│  │  ├─ sounds.rs             소리 파일 보관함 (복사·검사·삭제)
│  │  ├─ config/               설정 데이터 모양과 파일 저장
│  │  ├─ tray.rs, i18n.rs      메뉴 막대 아이콘·메뉴와 그 문구
│  │  ├─ shortcut.rs           전역 단축키
│  │  ├─ autostart.rs          로그인 시 자동 실행
│  │  ├─ window.rs, actions.rs 창 열기/숨기기, 공통 동작
│  │  └─ error.rs              오류 종류
│  ├─ icons/                   앱 아이콘, 메뉴 막대 아이콘
│  └─ tauri.conf.json          앱 이름, 창 크기, 번들 설정
│
├─ crates/app-watch/           앱 감지 모듈 (따로 테스트 가능)
│  └─ src/  watcher.rs(켜짐/꺼짐 감지) · identity.rs(앱 비교) · catalog/(앱 목록)
│
└─ .github/workflows/          ci.yml(검사), release.yml(설치 파일 만들기)
```

## 자주 고칠 곳

| 하고 싶은 것 | 고칠 곳 |
|---|---|
| 화면 문구 | `src/i18n/ko.ts` (+ `en.ts`, `ja.ts`) |
| 메뉴 막대 문구 | `src-tauri/src/i18n.rs` |
| 색·모서리·글꼴 | `src/styles/tokens.css` |
| 테마 추가 | `tokens.css`에 `:root[data-theme="..."]` 블록 + `src/theme.ts` |
| 앱 이름·번들 ID | `src-tauri/tauri.conf.json` 의 `productName`, `identifier` (배포 전에 꼭 본인 것으로) |
| 감지 간격 (기본 1초) | `src-tauri/src/lib.rs` 의 `WATCH_INTERVAL` |
| 받는 소리 형식 | `src-tauri/src/sounds.rs` 의 `SUPPORTED_EXTENSIONS` |
| 설정 항목 추가 | `src-tauri/src/config/mod.rs` `Settings` + `src/lib/types.ts` + 설정 창 |

설정 파일 위치 (문제 생기면 지우면 초기화됨):
- 맥: `~/Library/Application Support/com.appchime.app/config.json`, 소리 파일은 같은 폴더의 `sounds/`
- 윈도우: `%APPDATA%\com.appchime.app\config.json`, 소리 파일은 `%APPDATA%\com.appchime.app\sounds\`

## 동작 방식 메모

- **감지**: 1초마다 프로세스 목록을 비교합니다. CPU·메모리 정보는 읽지 않고 이름·경로만 읽어서 가볍습니다.
  같은 실행 파일이 여러 개 떠도 "0개→1개"만 켜짐, "1개→0개"만 꺼짐으로 보므로 소리가 반복되지 않습니다.
- **소리 파일**은 앱 보관함으로 복사해 쓰므로 원본을 옮겨도 계속 재생됩니다.
- **오디오 장치**는 재생할 때만 열고 5초 동안 조용하면 닫습니다. 이어폰을 바꿔도 다음 재생부터 새 장치로 나옵니다.

## 앞으로 하면 좋은 것

- 실제 앱 아이콘 표시 (지금은 이름 글자로 대신함, `src/components/AppAvatar.tsx`만 바꾸면 됨)
- 맥에서 시스템 알림(NSWorkspace) 방식으로 감지해 1초 지연 없애기
- 테마 선택, 강조색 선택 (설정 값 `theme`은 이미 있음)
- Microsoft Store 앱(UWP) 지원

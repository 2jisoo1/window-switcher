# SRS: 앱 전환 오버레이에서 Alt+`(백틱)로 역방향 선택 전환

문서 형식: IEEE/ISO/IEC 29148:2018 (System/Software Requirements Specification)
상태: Implemented (2026-06-29)
대상 소프트웨어: window-switcher v1.18.0 (fork)

---

## 1. Introduction

### 1.1 Purpose
window-switcher의 앱 전환 오버레이(`[switch-apps]`, 기본 `Alt+Tab`)는 `Alt`를 누른 채 `Tab`을 반복하면 선택이 **정방향(앞으로)** 으로만 순환한다. 역방향 이동은 `Shift+Tab`으로만 가능하다. 본 명세는 macOS의 `Cmd`+`` ` `` 동작을 흉내내어, **앱 전환 오버레이가 열려 있는 동안 `Alt`+`` ` ``(백틱)를 누르면 선택이 역방향으로 이동**하는 기능 요구사항을 정의한다.

### 1.2 Scope
- 포함: 앱 전환 오버레이가 활성(`IS_SWITCHING_APPS == true`)인 동안, switch-windows 핫키(기본 `Alt`+`` ` ``) 입력을 앱 선택의 역방향 순환으로 재해석.
- 제외: 오버레이가 닫혀 있을 때의 `Alt`+`` ` `` 동작(= 같은 앱의 창 전환, `[switch-windows]`)은 기존 그대로 유지한다.

### 1.3 Definitions
- **앱 전환 오버레이(App-switch overlay)**: `Alt+Tab`으로 열리는, 앱 아이콘을 가로로 나열한 레이어드 윈도우.
- **활성 상태**: 오버레이가 화면에 표시되어 선택 순환이 가능한 상태. 코드상 `keyboard.rs`의 `IS_SWITCHING_APPS` 플래그가 `true`인 구간.
- **switch-windows 핫키**: `[switch-windows]`의 핫키(기본 `Alt`+`` ` ``, scancode `0x29`).
- **switch-apps 핫키**: `[switch-apps]`의 핫키(기본 `Alt+Tab`, scancode `0x0f`).
- **역방향(reverse)**: 선택 인덱스를 감소(끝에서 처음으로 wrap)시키는 방향. 코드상 `WM_USER_SWITCH_APPS`의 `lparam == 1`.

### 1.4 References
- IEEE/ISO/IEC 29148:2018.
- sigoden/window-switcher 소스 (src/keyboard.rs, src/app.rs).
- 기존 명세: docs/specs/2026-06-22-app-names-under-icons.md.

---

## 2. Overall Description

### 2.1 Product Perspective
키보드 후킹(`WH_KEYBOARD_LL`, `keyboard.rs`)이 핫키 조합을 감지해 메인 윈도우로 사용자 메시지를 보낸다. 앱 선택의 정/역방향은 이미 `WM_USER_SWITCH_APPS`의 `lparam`(reverse 플래그)과 화살표 키 처리에 존재한다. 본 기능은 새 순환 로직을 추가하지 않고, 오버레이 활성 구간에서 switch-windows 핫키 입력을 기존 역방향 앱 순환 메시지로 라우팅하기만 한다.

### 2.2 User Characteristics
Alt+Tab으로 앱을 빠르게 오가며, macOS의 `Cmd`+`` ` `` 역방향 전환에 익숙한 Windows 사용자.

### 2.3 Constraints
- CON-1: Rust + Windows 링커 툴체인으로 재빌드가 필요하다(docs/BUILD.md).
- CON-2: `[switch-apps] enable = yes`일 때만 오버레이가 열리므로, 본 기능도 그 경우에만 동작한다.

---

## 3. Specific Requirements

### 3.1 Functional Requirements
- **REQ-F-1 (역방향 순환)**: 앱 전환 오버레이가 활성인 동안 switch-windows 핫키(기본 `Alt`+`` ` ``)를 누르면, 선택을 **역방향**으로 한 칸 이동한다(처음에서 누르면 마지막으로 wrap).
- **REQ-F-2 (방향 반전 옵션)**: 위 상황에서 `Shift`를 함께 누르면 **정방향**으로 이동한다(`Tab`/`Shift+Tab`의 방향 관례와 일치).
- **REQ-F-3 (오버레이 유지)**: 역방향 이동 입력은 오버레이를 닫지 않으며, `Alt` 해제 시 현재 선택 앱으로 전환이 확정된다(기존 확정 로직 재사용).
- **REQ-F-4 (입력 소모)**: 오버레이 활성 중의 `Alt`+`` ` `` 입력은 다른 앱으로 전파되지 않는다(후킹에서 소모).

### 3.2 Non-Functional Requirements
- **REQ-N-1 (회귀 없음 — 창 전환)**: 오버레이가 닫혀 있을 때 `Alt`+`` ` ``는 기존대로 같은 앱의 창 전환으로 동작한다.
- **REQ-N-2 (회귀 없음 — 비활성)**: `[switch-apps] enable = no`이면 본 기능은 동작하지 않으며 기존 동작에 영향이 없다.
- **REQ-N-3 (블랙리스트 무관)**: 오버레이 활성 중의 역방향 이동은 switch-windows 블랙리스트의 영향을 받지 않는다(앱 전환 동작이므로).

### 3.3 Acceptance Criteria
- AC-1: 앱 3개 이상을 띄우고 `Alt+Tab`으로 오버레이를 연 뒤 `Alt`를 유지한 채 `` ` ``를 누르면 선택이 한 칸 뒤로 이동한다.
- AC-2: 첫 항목에서 `Alt`+`` ` ``를 누르면 마지막 항목으로 wrap된다.
- AC-3: 오버레이 활성 중 `Alt+Shift`+`` ` ``는 선택을 앞으로 이동한다.
- AC-4: `` ` ``로 원하는 앱에 도달한 뒤 `Alt`를 떼면 그 앱으로 전환된다.
- AC-5: 오버레이를 열지 않은 상태의 `Alt`+`` ` ``는 여전히 같은 앱의 창 전환으로 동작한다(회귀 없음).

---

## 4. Verification
빌드 후 실제 실행하여 AC-1~AC-5를 수동 검증한다(전역 키 후킹/오버레이 특성상 자동화 곤란). 핫키 파싱 회귀는 기존 `config.rs` 단위 테스트로 확인한다.

---

## 5. Traceability (요구사항 ↔ 구현)

| 요구사항 | 구현 위치 |
| --- | --- |
| REQ-F-1, F-2 | `src/keyboard.rs` `keyboard_proc()` — `IS_SWITCHING_APPS && id==SWITCH_WINDOWS_HOTKEY_ID`일 때 `SWITCH_APPS_HOTKEY_ID` + reverse 메시지 전송 |
| REQ-F-3 | `src/app.rs` `WM_USER_SWITCH_APPS_DONE` → `do_switch_app()` (기존 확정 로직) |
| REQ-F-4 | `src/keyboard.rs` — 액션 메시지 전송 후 `return LRESULT(1)` |
| REQ-N-1, N-2 | `src/keyboard.rs` — 분기 조건이 `IS_SWITCHING_APPS`로 가드되어 오버레이 비활성 시 기존 경로 유지 |
| REQ-N-3 | `src/keyboard.rs` — 해당 분기는 블랙리스트(`IS_FOREGROUND_IN_BLACKLIST`)를 검사하지 않음 |

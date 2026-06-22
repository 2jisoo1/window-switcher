# SRS: Alt+Tab 오버레이에서 선택된 앱의 이름 표시

문서 형식: IEEE/ISO/IEC 29148:2018 (System/Software Requirements Specification)
상태: Implemented (2026-06-22)
대상 소프트웨어: window-switcher v1.18.0 (fork)

---

## 1. Introduction

### 1.1 Purpose
window-switcher의 앱 전환(`[switch-apps]`, 기본 `Alt+Tab`) 오버레이는 앱 아이콘만 가로로 나열한다. 본 명세는 **현재 선택(포커싱)된 앱의 이름을 그 아이콘 하단에 표시**하여, 아이콘만으로 구분이 어려운 경우에도 사용자가 어떤 앱이 선택돼 있는지 식별할 수 있도록 하는 기능 요구사항을 정의한다.

### 1.2 Scope
- 포함: 앱 전환 오버레이(`SwitchAppsState` 렌더링)에 선택 앱 이름 텍스트와 아이콘 간격을 추가.
- 제외: 같은 앱의 창 전환(`[switch-windows]`, `Alt+\``) 오버레이는 본 기능 대상이 아니다 (해당 기능은 오버레이 UI가 없음).

### 1.3 Definitions
- **앱(App)**: window-switcher가 실행파일(module path) 단위로 묶은 창 그룹. Chrome/Edge는 프로필별로 별도 그룹이 되며, 그룹 키는 `경로::프로필` 형태의 합성 문자열이다.
- **선택 앱(Focused app)**: 오버레이에서 현재 강조 표시된 항목(`SwitchAppsState.index`).
- **앱 이름(App name)**: 실행파일의 버전정보 `FileDescription` 문자열. 비어 있으면 확장자를 제거한 실행파일명으로 대체(fallback).
- **오버레이(Overlay)**: 전환 중 화면 중앙에 표시되는 레이어드 윈도우(`painter.rs`).

### 1.4 References
- IEEE/ISO/IEC 29148:2018.
- sigoden/window-switcher 소스 (src/app.rs, src/painter.rs, src/utils/window.rs).

---

## 2. Overall Description

### 2.1 Product Perspective
기존 GDI+ 기반 렌더러를 확장한다. 아이콘은 6배 확대 비트맵에 그린 뒤 축소(StretchBlt)하여 안티앨리어싱한다. 텍스트도 동일한 불투명 비트맵에 6배로 그린 뒤 함께 축소하여, 레이어드 윈도우의 알파를 해치지 않으면서 일관된 안티앨리어싱 화질을 얻는다.

### 2.2 User Characteristics
다수 앱을 Alt+Tab으로 빠르게 전환하는 Windows 사용자.

### 2.3 Constraints
- CON-1: Rust + Windows 링커 툴체인으로 재빌드가 필요하다(빌드 방법은 docs/BUILD.md 참고).
- CON-2: 레이어드 윈도우(per-pixel alpha) 특성상, 텍스트는 알파가 보존되는 방식으로 렌더링해야 한다(불투명 비트맵에 그려 합성).

---

## 3. Specific Requirements

### 3.1 Functional Requirements
- **REQ-F-1 (선택 앱 이름 표시)**: 오버레이에서 **선택된 앱의 이름만** 그 아이콘 하단에 표시한다. 선택되지 않은 아이콘에는 이름을 표시하지 않는다.
- **REQ-F-2 (이름 출처)**: 이름은 실행파일 `FileDescription`을 사용한다. 값이 없으면 확장자를 제거한 실행파일명을 사용한다. Chrome/Edge처럼 `경로::프로필` 형태의 합성 그룹 키는 `::` 앞의 실제 실행파일 경로로 정규화한 뒤 이름을 구한다(예: "Google Chrome").
- **REQ-F-3 (줄바꿈)**: 이름이 아이콘 열 너비를 초과하면 단어/문자 단위로 줄바꿈한다. 표시 높이를 한정하기 위해 최대 2줄로 제한하며, 초과분은 잘라낸다(clip).
- **REQ-F-4 (정렬)**: 이름 텍스트는 선택된 아이콘 열의 너비 안에서 수평 중앙에 정렬한다.
- **REQ-F-5 (선택 강조 유지)**: 기존의 선택 항목 강조 박스는 그대로 유지하며, 그 크기는 아이콘 셀 크기와 동일하다.
- **REQ-F-6 (클릭 좌표 정합성)**: 오버레이 높이(이름 영역) 및 열 간격 변경 후에도 마우스 클릭→앱 인덱스 매핑이 정확해야 한다.
- **REQ-F-7 (아이콘 간격)**: 아이콘 사이에 기본 12px(DPI 스케일 적용)의 간격을 둔다. 간격은 열 피치에만 더하며, 선택 강조 박스(아이콘 셀) 크기는 변하지 않는다.

### 3.2 Non-Functional Requirements
- **REQ-N-1 (테마)**: 텍스트 색은 라이트/다크 테마에 따라 배경과 대비되도록 한다.
- **REQ-N-2 (DPI)**: 폰트 크기와 간격은 기존 아이콘 크기와 동일한 DPI 스케일을 따른다.
- **REQ-N-3 (성능)**: 추가 렌더링으로 전환 응답성이 체감 저하되지 않아야 한다.
- **REQ-N-4 (회귀 없음)**: `[switch-apps] enable=no`거나 switch-windows 동작에는 영향이 없다.

### 3.3 Acceptance Criteria
- AC-1: 서로 다른 앱 여러 개를 띄우고 Alt+Tab 시, 선택된 아이콘 아래에만 그 앱 이름이 보인다.
- AC-2: 이름이 긴 앱은 아이콘 열 너비 안에서 최대 2줄로 줄바꿈되고, 그보다 길면 잘려서 표시된다.
- AC-3: 마우스로 특정 아이콘을 클릭하면 해당 앱으로 전환된다.
- AC-4: 라이트/다크 테마 모두에서 텍스트가 읽힌다.
- AC-5: 프로필이 적용된 Chrome도 "Google Chrome"처럼 정규화된 이름으로 보인다.
- AC-6: 아이콘 사이에 약 12px의 간격이 보이며, 선택 강조 박스 크기는 이전과 동일하다.

---

## 4. Verification
빌드 후 실제 실행하여 AC-1~AC-6을 수동 검증한다(GUI 오버레이 특성상 자동화 곤란).

---

## 5. Traceability (요구사항 ↔ 구현)

| 요구사항 | 구현 위치 |
| --- | --- |
| REQ-F-1, F-4 | `src/painter.rs` `draw_icons()` — 선택 인덱스 라벨만 `DrawTextW`(DT_CENTER) |
| REQ-F-2 | `src/utils/window.rs` `get_app_name()` / `get_file_description()` |
| REQ-F-3 | `src/painter.rs` — `DT_WORDBREAK`, 라벨 밴드 높이 `LABEL_LINES=2`로 클립 |
| REQ-F-5 | `src/painter.rs` `draw_icons()` — `scaled_icon_cell` 기준 강조 박스 |
| REQ-F-6 | `src/painter.rs` `find_clicked_app_index()` — `item_size`(열 피치)·`icon_cell` 반영 |
| REQ-F-7 | `src/painter.rs` `ICON_GAP_BASE`, `Coordinate`(item_size=icon_cell+gap) |
| REQ-N-1 | `src/painter.rs` `TEXT_DARK_COLOR`/`TEXT_LIGHT_COLOR` + 테마 분기 |
| REQ-N-2 | `src/painter.rs` `label_metrics()`, DPI 스케일 적용 |
| 상태 보관 | `src/app.rs` `SwitchAppsState.apps: Vec<(HICON, HWND, String)>` |

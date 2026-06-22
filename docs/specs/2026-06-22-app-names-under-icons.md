# SRS: Alt+Tab 오버레이 아이콘 하단 앱 이름 표시

문서 형식: IEEE/ISO/IEC 29148:2018 (System/Software Requirements Specification)
상태: Draft
작성일: 2026-06-22
대상 소프트웨어: window-switcher v1.18.0 (fork)

---

## 1. Introduction

### 1.1 Purpose
window-switcher의 앱 전환(`[switch-apps]`, 기본 `Alt+Tab`) 오버레이는 현재 앱 아이콘만 가로로 나열한다. 본 명세는 **각 아이콘 하단에 해당 앱의 이름을 표시**하여, 아이콘만으로 구분이 어려운 경우에도 사용자가 대상 앱을 식별할 수 있도록 하는 기능 요구사항을 정의한다.

### 1.2 Scope
- 포함: 앱 전환 오버레이(`SwitchAppsState` 렌더링)에 아이콘별 이름 텍스트 추가.
- 제외: 같은 앱의 창 전환(`[switch-windows]`, `Alt+\``) 오버레이는 본 기능 대상이 아니다 (해당 기능은 오버레이 UI가 없음).

### 1.3 Definitions
- **앱(App)**: window-switcher가 실행파일(module path) 단위로 묶은 창 그룹.
- **앱 이름(App name)**: 실행파일의 버전정보 `FileDescription` 문자열. 비어 있으면 확장자를 제거한 실행파일명으로 대체(fallback).
- **오버레이(Overlay)**: 전환 중 화면 중앙에 표시되는 레이어드 윈도우(`painter.rs`).

### 1.4 References
- IEEE/ISO/IEC 29148:2018.
- sigoden/window-switcher 소스 (src/app.rs, src/painter.rs, src/utils/window.rs).

---

## 2. Overall Description

### 2.1 Product Perspective
기존 GDI+ 기반 렌더러를 확장한다. 아이콘은 6배 확대 비트맵에 그린 뒤 축소(StretchBlt)하여 안티앨리어싱한다. 텍스트도 동일 비트맵에 확대 렌더링 후 축소하여 일관된 화질을 얻는다.

### 2.2 User Characteristics
다수 앱을 Alt+Tab으로 빠르게 전환하는 Windows 사용자.

### 2.3 Constraints
- CON-1: Rust + Windows 링커 툴체인으로 재빌드가 필요하다.
- CON-2: 레이어드 윈도우(per-pixel alpha) 특성상, 텍스트는 알파가 보존되는 방식으로 렌더링해야 한다(불투명 비트맵에 그려 합성).

---

## 3. Specific Requirements

### 3.1 Functional Requirements
- **REQ-F-1 (이름 표시)**: 오버레이의 각 아이콘 하단에 해당 앱의 이름을 표시한다.
- **REQ-F-2 (이름 출처)**: 이름은 실행파일 `FileDescription`을 사용한다. 값이 없으면 확장자를 제거한 실행파일명을 사용한다.
- **REQ-F-3 (줄바꿈)**: 이름이 아이콘 열 너비를 초과하면 단어/문자 단위로 줄바꿈한다. 표시 높이를 한정하기 위해 최대 2줄로 제한하고, 초과분은 말줄임(…) 처리한다.
- **REQ-F-4 (정렬)**: 이름 텍스트는 각 아이콘 열 중앙에 수평 정렬한다.
- **REQ-F-5 (선택 강조 유지)**: 기존의 선택 항목 강조 박스는 그대로 유지한다.
- **REQ-F-6 (클릭 좌표 정합성)**: 오버레이 높이 변경 후에도 마우스 클릭→앱 인덱스 매핑이 정확해야 한다.

### 3.2 Non-Functional Requirements
- **REQ-N-1 (테마)**: 텍스트 색은 라이트/다크 테마에 따라 배경과 대비되도록 한다.
- **REQ-N-2 (DPI)**: 폰트 크기는 기존 아이콘 크기와 동일한 DPI 스케일을 따른다.
- **REQ-N-3 (성능)**: 추가 렌더링으로 전환 응답성이 체감 저하되지 않아야 한다.
- **REQ-N-4 (회귀 없음)**: `[switch-apps] enable=no`거나 switch-windows 동작에는 영향이 없다.

### 3.3 Acceptance Criteria
- AC-1: 서로 다른 앱 4개 이상을 띄우고 Alt+Tab 시, 각 아이콘 아래 앱 이름이 보인다.
- AC-2: 이름이 긴 앱(예: 긴 제품명)은 2줄까지 줄바꿈되고 그 이상은 …로 잘린다.
- AC-3: 마우스로 특정 아이콘을 클릭하면 해당 앱으로 전환된다.
- AC-4: 라이트/다크 테마 모두에서 텍스트가 읽힌다.

---

## 4. Verification
빌드 후 실제 실행하여 AC-1~AC-4를 수동 검증한다(GUI 오버레이 특성상 자동화 곤란).

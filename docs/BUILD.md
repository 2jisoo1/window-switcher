# 소스에서 빌드하기 (Windows)

이 포크는 `window-switcher` 원본에 **Alt+Tab 오버레이에서 선택된 앱의 이름을
아이콘 아래에 표시**하는 기능을 추가한 버전입니다. 이 문서는 소스를 직접 빌드해
`window-switcher.exe`를 만드는 방법을 정리합니다.

> 결론부터: **Rust 툴체인 + C 링커**가 있으면 빌드됩니다. 아래 두 경로 중 하나를
> 고르세요. 이미 Visual Studio(C++)가 있으면 **경로 A(MSVC)**, 가벼운 설치를
> 원하면 **경로 B(GNU)** 를 권장합니다.

---

## 0. 공통 준비

- Windows 10/11 (64-bit)
- `git`, `winget` (Windows 기본 제공)
- 저장소 클론

```powershell
git clone https://github.com/2jisoo1/window-switcher.git
cd window-switcher
```

---

## 경로 A — MSVC 툴체인 (표준)

Windows에서 Rust의 기본 타깃입니다. C++ 빌드 도구(링커 `link.exe`)와
Windows SDK(리소스 컴파일러 `rc.exe`)가 필요합니다.

1. **Visual Studio Build Tools 설치** — "C++를 사용한 데스크톱 개발"
   워크로드를 선택해 설치합니다(이미 Visual Studio가 있고 C++ 워크로드를
   설치했다면 건너뜁니다).

   ```powershell
   winget install --id Microsoft.VisualStudio.2022.BuildTools -e
   ```
   설치 관리자가 뜨면 **"C++를 사용한 데스크톱 개발"** 워크로드를 체크하고 설치하세요.

2. **Rust 설치** (기본 타깃이 `x86_64-pc-windows-msvc`)

   ```powershell
   winget install --id Rustlang.Rustup -e
   ```
   설치 후 새 터미널을 열어 `cargo --version` 이 동작하는지 확인합니다.

3. **빌드**

   ```powershell
   cargo build --release
   ```

리소스(아이콘/버전정보)는 SDK의 `rc.exe`로 컴파일되므로 별도 도구가 필요 없습니다.

---

## 경로 B — GNU 툴체인 (가벼운 설치, 이 포크에서 검증된 경로)

Visual Studio를 설치하지 않고 빌드하는 방법입니다. 링커는 Rust GNU 툴체인에
포함되지만, 리소스 컴파일에 쓰이는 **`windres`** 는 별도 MinGW-w64에서 가져옵니다.

1. **Rust + GNU 툴체인 설치**

   ```powershell
   winget install --id Rustlang.Rustup -e
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   rustup component add rust-mingw   # dlltool 등 링킹 보조 포함
   ```

2. **MinGW-w64 설치** (`windres`, `dlltool` 제공) — WinLibs(MSVCRT 런타임)

   ```powershell
   winget install -e --id BrechtSanders.WinLibs.POSIX.MSVCRT
   ```

3. **MinGW `bin`을 PATH에 추가한 채로 빌드**

   WinLibs는 winget 패키지 폴더에 설치됩니다. 해당 `mingw64\bin`을 PATH 앞에
   놓고 빌드합니다(현재 세션 한정):

   ```powershell
   $mingw = (Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Recurse -Filter windres.exe |
             Select-Object -First 1).Directory.FullName
   $env:Path = "$env:USERPROFILE\.cargo\bin;$mingw;$env:Path"

   cargo build --release
   ```

   > PATH에 영구 등록하려면 시스템 환경변수에 위 `mingw64\bin` 경로를 추가하세요.

---

## 결과물 실행

빌드 산출물:

```
target\release\window-switcher.exe
```

- 더블클릭하면 트레이 아이콘으로 실행됩니다(설치 불필요, 단일 실행파일).
- 기존 실행본을 교체할 때는 **먼저 실행 중인 인스턴스를 종료**한 뒤 exe를 덮어쓰세요.

```powershell
Get-Process window-switcher -ErrorAction SilentlyContinue | Stop-Process -Force
Copy-Item target\release\window-switcher.exe "<배포 위치>\window-switcher.exe" -Force
```

---

## 설정 (`window-switcher.ini`)

실행파일과 같은 폴더의 `window-switcher.ini`로 동작을 조정합니다(시작 시 1회 로드되므로
수정 후에는 **재시작** 필요).

- `Alt+\`` (백틱): **같은 앱의 창들** 사이 전환 (`[switch-windows]`)
- `Alt+Tab`: **앱 간 전환 오버레이** — 이 포크의 이름 표시 기능이 여기서 동작합니다.
  사용하려면 `[switch-apps]` 의 `enable = yes` 로 설정하세요(기본 `no`).

```ini
[switch-apps]
enable = yes
hotkey = alt+tab
```

---

## 이름 표시 동작 (이 포크의 추가 기능)

- 표시되는 이름은 실행파일의 **FileDescription**(버전정보)입니다. 비어 있으면 exe 파일명으로
  대체됩니다. Chrome/Edge처럼 프로필별로 묶이는 경우 실제 exe 기준("Google Chrome")으로 보입니다.
- **선택된 아이콘 아래**에만, 그 아이콘 열 너비 안에서 표시되며 길면 두 줄로 줄바꿈됩니다.
- 폰트 크기/줄 수/아이콘 간격 등은 `src/painter.rs` 상단 상수로 조정합니다:
  `LABEL_FONT_SIZE_BASE`, `LABEL_LINES`, `ICON_GAP_BASE` 등.

---

## 문제 해결

| 증상 | 원인 / 해결 |
| --- | --- |
| `error calling dlltool 'dlltool.exe': program not found` | GNU 경로에서 `rustup component add rust-mingw` 실행, 또는 MinGW `bin`을 PATH에 추가 |
| `windres` 관련 빌드 실패 | GNU 경로에서 MinGW-w64(WinLibs) 설치 후 `windres.exe`가 PATH에 있는지 확인 |
| `link.exe`/`rc.exe` 없음 | MSVC 경로에서 VS Build Tools의 "C++ 데스크톱 개발" 워크로드 설치 |
| `cargo`를 못 찾음 | 설치 후 **새 터미널**을 열거나 `%USERPROFILE%\.cargo\bin`을 PATH에 추가 |
